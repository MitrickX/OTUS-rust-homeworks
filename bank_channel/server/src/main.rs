use server::server::handler::{handle, Context};
use std::io::{BufReader, Write};
use std::net::TcpListener;
use std::sync::{Arc, RwLock};

type Result<T> = std::result::Result<T, Box<dyn std::error::Error>>;

const ADDR: &str = "127.0.0.1:1337";

fn main() -> Result<()> {
    let listener = TcpListener::bind(ADDR)?;

    println!("Listening on {}", listener.local_addr()?);

    let original_lock_context = Arc::new(RwLock::new(Context::default()));

    for stream in listener.incoming() {
        let stream = stream?;

        let lock_context = Arc::clone(&original_lock_context);
        std::thread::spawn(move || loop {
            let mut reader = BufReader::new(&stream);
            let mut writer = stream.try_clone().unwrap();

            writer
                .write_all(
                    "Welcome to bank application\nPrint 'help' and press Enter to see the list of commands\n".as_bytes(),
                )
                .unwrap();

            let mut terminal = std::io::stdout();
            let lock_context = Arc::clone(&lock_context);

            match handle(lock_context, &mut reader, &mut writer, &mut terminal) {
                Ok(_) => break,
                Err(e) => println!("Error: {}", e),
            };
        });
    }

    Ok(())
}
