use std::{convert::TryInto, io::Read, thread, time::Duration};

mod protos;

// logging
extern crate pretty_env_logger;
#[macro_use]
extern crate log;

use protobuf::Message;
use protos::communique::{ButtonPushed, DisplayText};

fn main() {
    pretty_env_logger::init();
    info!("Starting up Management App");
    let ports = serialport::available_ports().expect("No Ports Found");
    let mut port = serialport::new("/dev/ttyUSB0", 115200)
        .timeout(Duration::from_millis(1000))
        .flow_control(serialport::FlowControl::None)
        .stop_bits(serialport::StopBits::One)
        .parity(serialport::Parity::None)
        .open()
        .expect("Failed to open port");

    info!(
        "Sitting in main loop, waiting for data to come through on {}",
        "/dev/ttyUSB0"
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
                    match bp.get_number() {
                        0 => {
                            println!("button1");
                        }
                        1 => {
                            println!("button2");
                        }
                        2 => {
                            println!("button3");
                        }
                        _ => {
                            error!("Unable to parse protobuf reply. Expected a number within range but got {}", bp.get_number());
                        }
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
