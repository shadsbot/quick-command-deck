use std::{convert::TryInto, fs, io::Read, path::PathBuf, str::FromStr, thread, time::Duration};

mod protos;

// logging
extern crate pretty_env_logger;
#[macro_use]
extern crate log;

use protobuf::Message;
use protos::communique::{ButtonPushed, DisplayText};

use toml::Value;

#[derive(Clone, Debug)]

struct Button {
    id: u8,
    command: Option<String>,
    log_message: Option<String>,
    label: Option<String>,
    icon: Option<Vec<u8>>,
}

struct Config {
    port: PathBuf,
    baudrate: u32,
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
        })
        .collect();

    ParsedToml {
        buttons: buttons,
        config: config,
    }
}

fn main() {
    pretty_env_logger::init();
    info!("Grabbing options from config file");
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
    loop {
        let bytes_to_read = port.bytes_to_read().unwrap();
        if bytes_to_read > 0 {
            info!("Heard {} bytes", bytes_to_read);
            let mut serial_buf: Vec<u8> = vec![0; bytes_to_read.try_into().unwrap()];
            port.read(&mut serial_buf).expect("No Data");
            match ButtonPushed::parse_from_bytes(serial_buf.as_mut_slice()) {
                Ok(bp) => {
                    info!("Button recognized as {}", bp.get_number());
                    let results: Vec<Button> = options
                        .buttons
                        .clone()
                        .into_iter()
                        .filter(|this| this.id == bp.get_number() as u8)
                        .map(|this| {
                            info!(
                                "{}",
                                this.log_message
                                    .unwrap_or(format!("Button match for {}", this.id))
                            );
                            if this.command.is_some() {
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
                                let command = this.command.clone().unwrap();
                                // toml wraps the command in quotes, which screws with the
                                // std::process::Command::new().args call
                                let command = &command[1..command.len()-1];
                                #[cfg(target_family = "windows")]
                                let cmd_out = std::process::Command::new("cmd")
                                    .args(["/C", command])
                                    .output()
                                    .expect(format!("Something went wrong executing your command. Tried `{}`", this.command.clone().unwrap()).as_str());
                                #[cfg(target_family = "unix")]
                                let cmd_out = std::process::Command::new("sh")
                                    // .arg("-c")
                                    .args(&["-c", command])
                                    // .arg(this.command.clone().unwrap())
                                    .output()
                                    .and_then(|r| match r.status.success() {
                                        true => Ok(r),
                                        false => {
                                            error!("Something went wrong executing your command.");
                                            error!("{}", String::from_utf8_lossy(&r.stderr));
                                            Err(std::io::Error::new(std::io::ErrorKind::InvalidData, "Error executing command"))
                                        },
                                    }).unwrap();
                                info!(
                                    "Executed `{}`, which returned `{}`",
                                    command,
                                    cmd_out.status
                                )
                            }
                            todo!("Implement the protobuf response to the microcontroller");
                        })
                        .collect();
                    if results.len() == 0 {
                        error!(
                            "Unable to parse protobuf reply. ID {} not found",
                            bp.get_number()
                        );
                    }
                }
                Err(e) => {
                    warn!("Unable to parse protobuf from bytes. Is this the wrong device? More info: {}", e);
                }
            }
            port.clear(serialport::ClearBuffer::Input)
                .expect("Failed to discard buffer");
        }

        thread::sleep(Duration::from_millis(100));
    }
}
