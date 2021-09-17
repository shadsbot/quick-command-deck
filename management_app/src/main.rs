use std::{
    convert::TryInto,
    io::{self, Read},
    sync::mpsc,
    thread,
    time::Duration,
};
// use serial::SerialPort;
// use serial::prelude::*;

mod protos;
use protobuf::Message;
use protos::communique::{ButtonPushed, DisplayText};

fn main() {
    let ports = serialport::available_ports().expect("No Ports Found");
    let mut port = serialport::new("/dev/ttyUSB0", 115200)
        .timeout(Duration::from_millis(1000))
        .flow_control(serialport::FlowControl::None)
        .stop_bits(serialport::StopBits::One)
        .parity(serialport::Parity::None)
        .open()
        .expect("Failed to open port");

    loop {
        println!(
            "Bytes to read: {}",
            port.bytes_to_read().expect("Error calling bytes_to_read")
        );
        let bytes_to_read = port.bytes_to_read().unwrap();
        if bytes_to_read > 0 {
            let mut serial_buf: Vec<u8> = vec![0; bytes_to_read.try_into().unwrap()];

            port.read(&mut serial_buf).expect("No Data");
            match ButtonPushed::parse_from_bytes(serial_buf.as_mut_slice()) {
                Ok(bp) => {
                    println!("{:?}", bp.get_number());
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
                            println!("Unable to parse protobuf reply. Expected number (0..X).");
                        }
                    }
                }
                Err(e) => {
                    println!("Unable to parse protobuf from bytes. Maybe this is the wrong device? More info:");
                    println!("{}", e);
                }
            }
            port.clear(serialport::ClearBuffer::Input)
                .expect("Failed to discard buffer");
        }

        thread::sleep(Duration::from_millis(100));
    }
}
