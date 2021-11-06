use std::{convert::TryInto, io::Read, sync::mpsc, thread, time::Duration};

mod protos;

// logging
extern crate pretty_env_logger;
#[macro_use]
extern crate log;

use protobuf::{Message, RepeatedField};
use protos::communique::{ButtonPushed, DisplayText};

mod config_parser;
use crate::config_parser::read_toml;

#[cfg(feature = "dbus")]
mod optional_features;

#[derive(Clone, Debug)]

pub(crate) struct Button {
    id: u8,
    command: Option<String>,
    log_message: Option<String>,
    report_message: Option<Vec<String>>,
}

impl Button {
    fn execute_command(&self) {
        // Command can be manipulated so that the target process is
        // spawned on the right shell for the right system, however
        // it may be easier for the user to understand what's going
        // on if what they put in the config.toml is exactly what's
        // going to be executed.
        //
        // NB: You are straight up running commands through a shell
        // for this part. That's kind of the entire point, but it
        // goes without saying that this is a potential security
        // concern and should be treated carefully.
        if self.command.is_some() {
            // toml wraps the command in quotes, which screws with
            // the std::process::Command::new().args call
            let command = self.command.as_ref().unwrap().as_str();

            #[cfg(target_family = "windows")]
            let cmd_out = std::process::Command::new("cmd")
                .args(["/C", command])
                .output()
                .expect(
                    format!(
                        "Something went wrong executing your command. Tried `{}`",
                        self.command.clone().unwrap()
                    )
                    .as_str(),
                );

            #[cfg(target_family = "unix")]
            let cmd_out = std::process::Command::new("sh")
                .args(&["-c", command])
                .output()
                .and_then(|r| match r.status.success() {
                    true => Ok(r),
                    false => {
                        error!("Something went wrong executing your command.");
                        error!("{}", String::from_utf8_lossy(&r.stderr));
                        Err(std::io::Error::new(
                            std::io::ErrorKind::InvalidData,
                            "Error executing command",
                        ))
                    }
                })
                .unwrap();

            info!("Executed `{}` which returned `{}`", command, cmd_out.status);
        }
    }
    fn build_response(&self, notif_time_ms: u32) -> DisplayText {
        let mut msg = DisplayText::new();
        msg.set_line(RepeatedField::from_vec(
            self.report_message.clone().unwrap(),
        ));
        msg.set_brightness(150);
        msg.set_duration_ms(notif_time_ms.try_into().unwrap_or_default());
        return msg;
    }
}

fn main() {
    pretty_env_logger::init();
    trace!("Grabbing options from config file");
    let options = read_toml();
    info!("Starting up Management App");
    let _ports = serialport::available_ports().expect("No Ports Found");
    let mut port = serialport::new(
        options.config.port.to_str().unwrap_or("config.toml"),
        options.config.baudrate,
    )
    .timeout(Duration::from_millis(1000))
    .flow_control(serialport::FlowControl::None)
    .stop_bits(serialport::StopBits::One)
    .parity(serialport::Parity::None)
    .open()
    .expect("Failed to open port");

    info!(
        "Sitting in main loop, waiting for data to come through on {}",
        options.config.port.to_str().unwrap_or("config.toml")
    );

    // Talk between threads to send DisplayText from anywhere to microcontroller
    let (dptx, dprx) = mpsc::channel::<DisplayText>();
    let (bptx, bprx) = mpsc::channel::<ButtonPushed>();

    // https://doc.rust-lang.org/std/sync/mpsc/struct.Sender.html
    let dptx_button_reply = dptx.clone();

    // Communication thread
    // Responsible for talking to the microcontroller
    let comms = thread::spawn(move || loop {
        // Got a message from the microcontroller, attempt to route it
        let bytes_to_read = port.bytes_to_read().unwrap();
        if bytes_to_read > 0 {
            info!("Heard {} bytes", bytes_to_read);
            let mut serial_buf: Vec<u8> = vec![0; bytes_to_read.try_into().unwrap()];
            // Potentially blocking
            port.read(&mut serial_buf).expect("No Data");
            trace!(
                "Got: {:?}",
                String::from_utf8_lossy(serial_buf.clone().as_mut_slice())
            );
            match ButtonPushed::parse_from_bytes(serial_buf.as_mut_slice()) {
                Ok(bp) => {
                    info!("Button recognized as {}", bp.get_number());
                    bptx.send(bp).ok();
                }
                Err(e) => {
                    warn!("Unable to parse protobuf from bytes. Is this the wrong device? More info: {}", e);
                }
            }
            port.clear(serialport::ClearBuffer::Input)
                .expect("Failed to discard buffer");
        }

        // Got a message locally, attempt to pass it through
        let attempt = dprx.try_recv();
        if !attempt.is_err() {
            let attempt = attempt.unwrap();
            match port
                .write(attempt.write_to_bytes().unwrap().as_mut_slice())
                .is_ok()
            {
                true => trace!("Sent message to microcontroller"),
                false => warn!(
                    "Unable to send message to microcontroller! Attempted to send: {:?}",
                    attempt.write_to_bytes().unwrap()
                ),
            }
        }

        // Don't run to death
        thread::sleep(Duration::from_millis(100));
    });

    // Button Handling
    // When a button is pushed, get a message from the Communication thread
    // and handle it here. Then, send a response back.
    let buttons = thread::spawn(move || loop {
        // Blocks until we get a message
        let msg = bprx.recv();
        if !msg.is_err() {
            let msg = msg.unwrap();
            let results: Vec<Button> = options
                .buttons
                .clone()
                .into_iter()
                .filter(|this| this.id == msg.get_number() as u8)
                .map(|this| {
                    info!(
                        "{}",
                        this.log_message
                            .clone()
                            .unwrap_or(format!("Button match for {}", this.id))
                    );
                    this.execute_command();
                    if options.config.send_completed_notifs {
                        trace!("Sending response message");
                        // cdiener fixme
                        dptx_button_reply.send(this.build_response(500)).ok();
                    }
                    return this;
                })
                .collect();
            if results.len() == 0 {
                error!(
                    "Unable to parse protobuf reply. ID {} not found.",
                    msg.get_number()
                );
            }
        }
    });

    // Messages from dbus
    // Example usage: If a chat message is left unread for a while,
    // send it to be physically displayed
    #[cfg(feature = "dbus")]
    {
        let dbus_message_sender = dptx.clone();
        let dbus = thread::spawn(move || {
            let conn = optional_features::dbus_integration::listener::connect(dbus_message_sender);
            if conn.is_err() {
                error!("Cannot connect to dbus. {}", conn.err().unwrap())
            }
        });
        dbus.join().expect("Could not connect to dbus");
    }

    comms
        .join()
        .expect("Communication issue occured with controller.");
    buttons
        .join()
        .expect("Unknown issue with primary button handling thread.");
}
