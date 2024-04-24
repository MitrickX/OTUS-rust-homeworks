use std::io::{BufRead, BufReader, ErrorKind, Write};
use std::net::TcpStream;
use std::str::from_utf8;

const ADDR: &'static str = "127.0.0.1:1337";

fn main() {
    let stream = TcpStream::connect(ADDR).unwrap();
    let _ = stream.set_nonblocking(true);

    let mut reader = BufReader::new(stream.try_clone().unwrap());
    let mut writer: TcpStream = stream;

    let mut output = std::io::stdout();
    let input = std::io::stdin();

    std::thread::spawn(move || loop {
        let mut buf = Vec::new();
        match reader.read_until(b'\n', &mut buf) {
            Ok(bytes_num) => {
                let _ = output.write_all(&buf[..bytes_num]);
                output.flush().unwrap();

                let msg = from_utf8(&buf[..bytes_num]).unwrap().to_owned();
                if msg.trim() == "Bye bye" {
                    std::process::exit(0);
                }
            }
            Err(e) if e.kind() == ErrorKind::WouldBlock => std::thread::yield_now(),
            Err(e) => panic!("Error happened: {}", e),
        }
    });

    loop {
        let mut msg: String = String::new();
        input.read_line(&mut msg).unwrap();
        if let Err(e) = writer.write_all(msg.as_bytes()) {
            panic!("Error happened: {}", e);
        }
    }
}
