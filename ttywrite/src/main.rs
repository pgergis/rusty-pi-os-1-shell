extern crate serial;
extern crate structopt;
extern crate xmodem;
#[macro_use] extern crate structopt_derive;

use std::path::PathBuf;
use std::time::Duration;

use structopt::StructOpt;
use serial::core::{CharSize, BaudRate, StopBits, FlowControl, SerialDevice, SerialPortSettings};
use xmodem::{Xmodem, Progress};

mod parsers;

use parsers::{parse_width, parse_stop_bits, parse_flow_control, parse_baud_rate};

#[derive(StructOpt, Debug)]
#[structopt(about = "Write to TTY using the XMODEM protocol by default.")]
struct Opt {
    #[structopt(short = "i", help = "Input file (defaults to stdin if not set)", parse(from_os_str))]
    input: Option<PathBuf>,

    #[structopt(short = "b", long = "baud", parse(try_from_str = "parse_baud_rate"),
                help = "Set baud rate", default_value = "115200")]
    baud_rate: BaudRate,

    #[structopt(short = "t", long = "timeout", parse(try_from_str),
                help = "Set timeout in seconds", default_value = "10")]
    timeout: u64,

    #[structopt(short = "w", long = "width", parse(try_from_str = "parse_width"),
                help = "Set data character width in bits", default_value = "8")]
    char_width: CharSize,

    #[structopt(help = "Path to TTY device", parse(from_os_str))]
    tty_path: PathBuf,

    #[structopt(short = "f", long = "flow-control", parse(try_from_str = "parse_flow_control"),
                help = "Enable flow control ('hardware' or 'software')", default_value = "none")]
    flow_control: FlowControl,

    #[structopt(short = "s", long = "stop-bits", parse(try_from_str = "parse_stop_bits"),
                help = "Set number of stop bits", default_value = "1")]
    stop_bits: StopBits,

    #[structopt(short = "r", long = "raw", help = "Disable XMODEM")]
    raw: bool,
}

fn send_data<T: std::io::Read, U: std::io::Write + std::io::Read>(buf_in: &mut T, buf_out: &mut U, raw: bool) -> Result<(u64),std::io::Error> {
    if !raw {
        let result = Xmodem::transmit(buf_in, buf_out);
        match result {
            Ok(x) => Ok(x as u64),
            Err(y) => Err(y)
        }
    } else {
        std::io::copy(buf_in, buf_out)
    }
}

fn main() {
    use std::fs::File;
    use std::io::{self, BufReader, BufRead};

    let opt = Opt::from_args();
    let mut serial = serial::open(&opt.tty_path).expect("path points to invalid TTY");

    // FIXME: Implement the `ttywrite` utility.

    // Update Settings From Args
    let mut tty_settings = serial.read_settings().expect("Failed to read settings");
    tty_settings.set_baud_rate(opt.baud_rate).expect("Failed baud rate update");
    tty_settings.set_char_size(opt.char_width);
    tty_settings.set_flow_control(opt.flow_control);
    tty_settings.set_stop_bits(opt.stop_bits);
    serial.write_settings(&tty_settings).expect("Failed settings update");
    serial.set_timeout(Duration::new(opt.timeout, 0)).expect("Invalid timeout");

    // Read & Write
    match opt.input {
        Some(path) => send_data(&mut File::open(path).expect("Invalid filepath"), &mut serial, opt.raw),
        None => send_data(&mut io::stdin(), &mut serial, opt.raw)
    }.expect("Send data failed");
}
