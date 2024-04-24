use server::server::handler::{handle, Context};
use std::io::BufReader;
use std::net::TcpListener;

type Result<T> = std::result::Result<T, Box<dyn std::error::Error>>;

const ADDR: &'static str = "127.0.0.1:1337";

// TODO: support multi clients (non-blocking serving)

fn main() -> Result<()> {
    let listener = TcpListener::bind(ADDR)?;

    println!("Listening on {}", listener.local_addr()?);

    let mut context = Context {
        banks: Vec::new(),
        current_bank: 0,
    };

    for stream in listener.incoming() {
        let stream = stream?;

        let mut reader = BufReader::new(&stream);
        let mut writer = stream.try_clone().unwrap();
        let mut terminal = std::io::stdout();
        handle(&mut context, &mut reader, &mut writer, &mut terminal)?;
    }

    Ok(())
}
