use server::server::handler::{handle, Context};
use std::io::BufReader;
use std::net::TcpListener;
use std::sync::{Arc, RwLock};

type Result<T> = std::result::Result<T, Box<dyn std::error::Error>>;

const ADDR: &str = "127.0.0.1:1337";

// TODO: support multithreads (multiclients)
// TODO: support help

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
