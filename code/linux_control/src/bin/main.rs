extern crate structopt;
extern crate serialport;
extern crate vumeter_lib;
use structopt::StructOpt;
use std::path::PathBuf;
use serialport::{SerialPortSettings,DataBits,FlowControl,StopBits,Parity};
use std::time::Duration;
use std::time::Instant;

use vumeter_lib::protocol as pkt;


#[derive(StructOpt,Debug)]
/// Interact with tube boards
enum Command {
    /// Detect how many tube boards there are connected to the serial port
    ///
    /// Prints out the number on stdout
    Detect{
        #[structopt(parse(from_os_str))]
        serial_port:PathBuf
    },
    /// Forward values from stdin to the tubes
    ///
    /// Reads float values in the range [0-1] from stdin and forwards them to the tubes.
    /// If you have 4 tubes, each line you pipe to stdin should look like
    /// "0.25 0.7 0.1 0.48" (without the quotes).
    Pipe{
        #[structopt(parse(from_os_str))]
        serial_port:PathBuf,
    }
}

const SERIAL_SETTINGS : SerialPortSettings = SerialPortSettings {
        baud_rate: 460800,
        data_bits: DataBits::Eight,
        flow_control: FlowControl::None,
        parity: Parity::None,
        stop_bits: StopBits::One,
        timeout: Duration::from_millis(100),
    };


fn detect(port : PathBuf, timeout : Duration) -> usize {
    let mut parser = pkt::ParseState::new();
    let mut port = serialport::open_with_settings(&port, &SERIAL_SETTINGS).unwrap();
    port.write_all(&pkt::Pkt::probe_pkt().format()).unwrap();
    port.flush().unwrap();
    let start = Instant::now();
    loop {
        let duration = Instant::now() - start;
        if duration > timeout {
            panic!("Detect timed out")
        }
        let mut buf = [0u8;6];
        let count = port.read(&mut buf).unwrap();
        for byte in buf[..count].iter().cloned() {
            if let Some(pkt) = parser.step_(byte) {
                match pkt {
                    pkt::Pkt::Set{ttl,brightness:pkt::Brightness::ZERO} => {
                        let diff = (pkt::Pkt::PROBE_TTL - ttl) as usize;
                        let num_tubes = diff / 2;
                        if (diff & 1) == 1 {
                            panic!("Got odd tube length response.");
                        } else {
                            return num_tubes;
                        }
                    },
                    _ => {
                        panic!("Invalid probe response");
                    }
                }
            }
        }
    }
}

use std::io; 
use std::io::prelude::*; 

fn pipe(port : PathBuf)  {
    let mut port = serialport::open_with_settings(&port, &SERIAL_SETTINGS).unwrap();
    io::stdin().lock().lines().for_each(|line| {
        let values : Vec<f32> = line.unwrap().split(" ").map(|s| s.parse::<f32>().unwrap()).collect();
        for (i,value) in values.iter().enumerate().rev() {
            let brightness = (value * pkt::Brightness::MAX_BRIGHTNESS as f32) as u16;
            let brightness = pkt::Brightness::new(brightness).unwrap();
            let pkt = pkt::Pkt::mk_pkt(i as i8, brightness).unwrap();
            port.write_all(&pkt.format()).unwrap();
            port.flush().unwrap();
        }
    });
}

fn main() {
    let command = Command::from_args();

    use Command::*;
    match command {
        Detect{serial_port}=>{println!("{}",detect(serial_port,Duration::from_millis(100)))},
        Pipe{serial_port}=>{pipe(serial_port)}
    }
}
