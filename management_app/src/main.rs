use std::{
    convert::TryInto, fs, io::Read, path::PathBuf, str::FromStr, sync::mpsc, thread, time::Duration,
};

mod protos;

// logging
extern crate pretty_env_logger;
#[macro_use]
extern crate log;

use protobuf::{Message, RepeatedField};
use protos::communique::{ButtonPushed, DisplayText};

use toml::Value;

#[cfg(feature = "dbus")]
mod optional_features;

#[derive(Clone, Debug)]

struct Button {
    id: u8,
    command: Option<String>,
    log_message: Option<String>,
    label: Option<String>,
    icon: Option<Vec<u8>>,
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
            let command = self.command.clone().unwrap();
            let command = &command[1..command.len() - 1];

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

struct Config {
    port: PathBuf,
    baudrate: u32,
    notif_time_ms: u32,
    send_completed_notifs: bool,
}

struct ParsedToml {
    buttons: Vec<Button>,
    config: Config,
}

fn read_toml() -> ParsedToml {
    let value = fs::read_to_string("src/config.toml")
        .expect("Couldn't read file")
        .parse::<Value>()
        .unwrap();
    let config: Config = Config {
        port: match PathBuf::from_str(value["config"]["port"].as_str().unwrap()) {
            Ok(path) => path,
            Err(err) => {
                error!("Couldn't find or open config file. {}", err);
                std::process::exit(1);
            }
        },
        baudrate: value["config"]["baudrate"]
            .as_integer()
            .unwrap_or_else(|| 115200) as u32,
        notif_time_ms: value["config"]["notif_time_ms"]
            .as_integer()
            .unwrap_or_else(|| 500) as u32,
        send_completed_notifs: value["config"]["send_completed_notifs"]
            .as_bool()
            .unwrap_or_default(),
    };

    let buttons: Vec<Button> = value["commands"]
        .as_table()
        .unwrap()
        .into_iter()
        .map(|tuple| tuple.1)
        .map(|key_pair| Button {
            id: match key_pair.get("id") {
                Some(v) => match v.as_integer() {
                    Some(v) => v as u8,
                    None => {
                        error!("Found a button without a valid ID! IDs can only be integers.");
                        panic!("Invalid ID found in config.toml.");
                    }
                },
                None => {
                    error!("Found a button without a valid ID! IDs can only be integers.");
                    panic!("Invalid ID found in config.toml.");
                }
            },
            command: match key_pair.get("command") {
                Some(v) => Some(v.to_string()),
                None => None,
            },
            log_message: match key_pair.get("log_message") {
                Some(v) => Some(v.to_string()),
                None => None,
            },
            label: match key_pair.get("label") {
                Some(v) => Some(v.to_string()),
                None => None,
            },
            icon: match key_pair.get("icon") {
                Some(v) => Some(
                    v.as_array()
                        .unwrap()
                        .into_iter()
                        .map(|index_value| index_value.as_integer().unwrap() as u8)
                        .collect(),
                ),
                None => None,
            },
            report_message: match key_pair.get("report_message") {
                Some(v) => {
                    let mut lines = Vec::<String>::new();
                    for line in v.as_array()
                        .unwrap()
                        {
                            lines.push(line.as_str().unwrap_or_default().to_owned());
                        };
                    Some(lines)
                }
                None => if config.send_completed_notifs {
                    panic!("Config issue: Notifs configured to send but no notif message found to send")
                } else {
                    None
                },
            },
        })
        .collect();

    ParsedToml {
        buttons: buttons,
        config: config,
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
                        dptx_button_reply
                            .send(this.build_response(options.config.notif_time_ms))
                            .ok();
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
