use std::io::Write;
use std::str::from_utf8;
use tokio::{
    io::{AsyncBufReadExt, AsyncWriteExt, BufReader},
    net::TcpStream,
};

const ADDR: &str = "127.0.0.1:1337";

#[tokio::main]
async fn main() {
    let stream = TcpStream::connect(ADDR).await.unwrap();
    let (reader, mut writer) = stream.into_split();
    let mut reader = BufReader::new(reader);

    tokio::spawn(async move {
        let input = std::io::stdin();
        loop {
            let mut msg: String = String::new();
            input.read_line(&mut msg).unwrap();
            if let Err(e) = writer.write_all(msg.as_bytes()).await {
                panic!("Error happened: {}", e);
            }
        }
    });

    let mut output = std::io::stdout();
    loop {
        let mut buf = Vec::new();
        match reader.read_until(b'\n', &mut buf).await {
            Ok(bytes_num) => {
                let _ = output.write_all(&buf[..bytes_num]);
                output.flush().unwrap();

                let msg = from_utf8(&buf[..bytes_num]).unwrap().to_owned();
                if msg.trim() == "Bye bye" {
                    std::process::exit(0);
                }
            }
            Err(e) => panic!("Error happened: {}", e),
        }
    }
}
