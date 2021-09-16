use std::{io::{self, Read}, sync::mpsc, thread, time::Duration};
// use serial::SerialPort;
// use serial::prelude::*;

fn main() {
    // SerialPorts
    let ports = serialport::available_ports().expect("No Ports Found");
    // print!("{:?}", ports);

    let mut port = serialport::new("/dev/ttyUSB0", 115200)
        .timeout(Duration::from_millis(10))
        .flow_control(serialport::FlowControl::None)
        .stop_bits(serialport::StopBits::One)
        .parity(serialport::Parity::None)
        .open().expect("Failed to open port");

    let mut serial_buf: Vec<u8> = vec![0; 1024];

    let chan_clear_buf = input_service();

    loop {
        println!("Bytes to read: {}", port.bytes_to_read().expect("Error calling bytes_to_read"));
        let bytes_to_read = port.bytes_to_read().unwrap();
        if bytes_to_read > 0 {
            port.read(serial_buf.as_mut_slice()).expect("No Data");
        
            for c in serial_buf.clone() {
                print!("{}", c as char);
            }
        }
        match chan_clear_buf.try_recv() {
            Ok(_) => {
                println!("Discarding buffer!");
                port.clear(serialport::ClearBuffer::Input).expect("Failed to discard buffer");
            }
            Err(mpsc::TryRecvError::Empty) => (),
            Err(mpsc::TryRecvError::Disconnected) => {
                println!("Stopping");
                break;
            }
        }

        thread::sleep(Duration::from_millis(100));
    }
}

// taken from https://gitlab.com/susurrus/serialport-rs/-/blob/master/examples/clear_input_buffer.rs
// 
// Including this makes this susceptible to being released under the MPL 2.0,
// but if this is what I think it is, it's only handling the keyboard input
// part of the code, and can safely be discarded in future commits.
fn input_service() -> mpsc::Receiver<()> {
    let (tx, rx) = mpsc::channel();
    thread::spawn(move || {
        let mut buffer = [0; 32];
        loop {
            // Blocking!
            match io::stdin().read(&mut buffer) {
                Ok(0) => {
                    drop(tx); // EOF, drop the channel and stop the thread
                    break;
                }
                Ok(_) => tx.send(()).unwrap(), // signal main to clear buffer
                Err(e) => panic!(e),
            }
        }
    });

    rx
}
