use server::bank::account::AccountID;
use std::error::Error;
use std::fmt;
use std::io::{BufRead, BufReader};
use std::net::{TcpListener, TcpStream};

enum Command {
    Register {
        balance: u64,
    },
    GetBalance {
        id: AccountID,
    },
    Deposit {
        id: AccountID,
        balance: u64,
    },
    Withdraw {
        id: AccountID,
        balance: u64,
    },
    Transfer {
        sender: AccountID,
        reciever: AccountID,
        ammount: u64,
    },
    ListAccountOperations {
        id: AccountID,
    },
    ListAllOperations,
    Quit,
}

#[derive(Debug, Clone)]
enum ParseError {
    EmptyCommand,
    RequireArguments(Vec<String>),
    InvalidArgumentUint(String, std::num::ParseIntError),
    InvalidArgumentAccountID(server::bank::account::Error),
    UnknownCommand(String),
}

impl fmt::Display for ParseError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            ParseError::EmptyCommand => write!(f, "empty command"),
            ParseError::RequireArguments(args) => {
                write!(f, "require arguments: {:?}", args)
            }
            ParseError::InvalidArgumentUint(name, e) => {
                write!(f, "invalid argument {name}: {e}")
            }
            ParseError::InvalidArgumentAccountID(e) => {
                write!(f, "invalid account id: {e}")
            }
            ParseError::UnknownCommand(command) => {
                write!(f, "unknown command: {command}")
            }
        }
    }
}

impl Error for ParseError {}

type Result<T> = std::result::Result<T, Box<dyn Error>>;

fn parse_argument_account_id(value: &str) -> Result<AccountID> {
    AccountID::parse_str(value).map_err(|e| ParseError::InvalidArgumentAccountID(e).into())
}

fn parse_argument_uint(name: &str, value: &str) -> Result<u64> {
    value
        .parse()
        .map_err(|e| ParseError::InvalidArgumentUint(name.to_string(), e).into())
}

fn parse_command(command: &str) -> Result<Command> {
    let parts: Vec<&str> = command.split(' ').map(|s| s.trim()).collect();
    if parts.is_empty() {
        return Err(ParseError::EmptyCommand.into());
    }

    let command = parts[0];

    match command {
        "get_balance" | "list_account_operations" => {
            if parts.len() < 2 {
                return Err(ParseError::RequireArguments(vec!["account_id".to_string()]).into());
            }

            match command {
                "get_balance" => Ok(Command::GetBalance {
                    id: parse_argument_account_id(parts[1])?,
                }),
                "list_account_operations" => Ok(Command::ListAccountOperations {
                    id: parse_argument_account_id(parts[1])?,
                }),
                _ => unreachable!(),
            }
        }
        "register" => {
            if parts.len() < 2 {
                return Err(ParseError::RequireArguments(vec!["balance".to_string()]).into());
            }

            Ok(Command::Register {
                balance: parse_argument_uint("balance", parts[1])?,
            })
        }
        "deposit" | "withdraw" => {
            if parts.len() < 3 {
                return Err(ParseError::RequireArguments(vec![
                    "account_id".to_string(),
                    "balance".to_string(),
                ])
                .into());
            }

            let id = parse_argument_account_id(parts[1])?;
            let balance = parse_argument_uint("ammount", parts[2])?;

            match command {
                "deposit" => Ok(Command::Deposit { id, balance }),
                "withdraw" => Ok(Command::Withdraw { id, balance }),
                _ => unreachable!(),
            }
        }
        "transfer" => {
            if parts.len() < 4 {
                return Err(ParseError::RequireArguments(vec![
                    "sender".to_string(),
                    "reciever".to_string(),
                    "ammount".to_string(),
                ])
                .into());
            }

            Ok(Command::Transfer {
                sender: parse_argument_account_id(parts[1])?,
                reciever: parse_argument_account_id(parts[2])?,
                ammount: parse_argument_uint("ammount", parts[3])?,
            })
        }
        "list_all_operations" => Ok(Command::ListAllOperations),
        "quit" => Ok(Command::Quit),
        _ => Err(ParseError::UnknownCommand(parts[0].to_string()).into()),
    }
}

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
