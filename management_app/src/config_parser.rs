use std::{fs, path::PathBuf, str::FromStr};

use toml::Value;

use crate::Button;

// since these aren't mutable, I don't really
// see a difference between having them be public
// and having getter functions for them. If this
// is wrong, please let me know.
pub(crate) struct Display {
    pub(crate) connected: bool,
    pub(crate) lines: u32,
    pub(crate) columns: u32,
    pub(crate) notif_time_ms: u32,
    pub(crate) brightness: u32,
}

pub(crate) struct Config {
    pub(crate) port: PathBuf,
    pub(crate) baudrate: u32,
    pub(crate) send_completed_notifs: bool,
    pub(crate) display: Display,
}

pub(crate) struct ParsedToml {
    pub(crate) buttons: Vec<Button>,
    pub(crate) config: Config,
}

pub(crate) fn read_toml() -> ParsedToml {
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

        display: Display {
            connected: value["config"]["display"]["connected"]
                .as_bool()
                .unwrap_or_default(),
            lines: value["config"]["display"]["lines"]
                .as_integer()
                .unwrap_or_default() as u32,
            columns: value["config"]["display"]["columns"]
                .as_integer()
                .unwrap_or_default() as u32,
            notif_time_ms: value["config"]["display"]["notif_time_ms"]
                .as_integer()
                .unwrap_or_else(|| 500) as u32,
            brightness: value["config"]["display"]["brightness"]
                .as_integer()
                .unwrap_or_else(|| 255) as u32,
        },

        send_completed_notifs: value["config"]["send_completed_notifs"]
            .as_bool()
            .unwrap_or_default(),
    };

    let mut buttons: Vec<Button> = vec![];
    for (index, command) in value["command"].as_array().unwrap().into_iter().enumerate() {
        buttons.push(Button {
            id: index as u8,
            command: match command.get("command") {
                Some(v) => Some(v.as_str().unwrap().to_string()),
                None => None,
            },
            log_message: match command.get("log_message") {
                Some(v) => Some(v.as_str().unwrap().to_string()),
                None => None,
            },
            report_message: match command.get("report_message") {
                Some(v) => Some(
                    v.as_array()
                        .unwrap()
                        .into_iter()
                        .map(|val| val.as_str().unwrap().to_string())
                        .collect(),
                ),
                None => if config.send_completed_notifs {
                    panic!("Config issue: Notifs configured to send but no notif message found to send")
                } else {
                    None
                },
            },
        });
    }

    ParsedToml {
        buttons: buttons,
        config: config,
    }
}
