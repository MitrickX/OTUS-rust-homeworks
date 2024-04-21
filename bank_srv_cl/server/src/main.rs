use server::server::command::{parse_command, Command};
use std::io::{BufRead, BufReader};
use std::net::{TcpListener, TcpStream};

type Result<T> = std::result::Result<T, Box<dyn std::error::Error>>;

fn process(stream: TcpStream) -> Result<()> {
    let mut reader = BufReader::new(&stream);
    loop {
        let mut line = String::new();
        match reader.read_line(&mut line) {
            Ok(n) => {
                if n == 0 {
                    println!("Client disconnected");
                    break;
                }

                let command = parse_command(&line)?;
                match command {
                    Command::Register { balance } => {
                        println!("Register balance: {}", balance);
                    }
                    Command::GetBalance { id } => {
                        println!("Get balance: {}", id);
                    }
                    Command::Deposit { id, balance } => {
                        println!("Deposit: {id}, {balance}");
                    }
                    Command::Withdraw { id, balance } => {
                        println!("Withdraw: {id}, {balance}");
                    }
                    Command::Transfer {
                        sender,
                        reciever,
                        ammount,
                    } => {
                        println!("Transfer: {sender}, {reciever}, {ammount}");
                    }
                    Command::ListAccountOperations { id } => {
                        println!("List account operations: {}", id);
                    }
                    Command::ListAllOperations => {
                        println!("List all operations");
                    }
                    Command::Quit => {
                        println!("Client quited");
                        break;
                    }
                }
            }
            Err(e) if e.kind() == std::io::ErrorKind::InvalidData => {
                // just ignore invalid data
                continue;
            }
            Err(e) => return Err(e.into()),
        };
    }

    Ok(())
}

fn main() -> Result<()> {
    let listener = TcpListener::bind("127.0.0.1:1337")?;

    println!("Listening on {}", listener.local_addr()?);

    for stream in listener.incoming() {
        let stream = stream?;
        process(stream)?;
    }

    Ok(())
}
