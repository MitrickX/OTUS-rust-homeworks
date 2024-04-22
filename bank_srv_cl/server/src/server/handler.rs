use crate::bank::account::Account;
use crate::bank::Bank;
use crate::server::command::{parse_command, Command};

pub struct Context {
    pub banks: Vec<Bank>,
    pub current_bank: usize,
}

type Result<T> = std::result::Result<T, Box<dyn std::error::Error>>;

pub fn handle<R: std::io::BufRead, W: std::io::Write>(
    context: &mut Context,
    reader: &mut R,
    writer: &mut W,
) -> Result<()> {
    loop {
        let mut line = String::new();
        match reader.read_line(&mut line) {
            Ok(n) => {
                if n == 0 {
                    println!("Client disconnected");
                    break;
                }

                match parse_command(&line) {
                    Ok(command) => {
                        match command {
                            Command::NewBank => {
                                context.banks.push(Bank::new());
                                context.current_bank = context.banks.len() - 1;
                                writer.write_all(
                                    format!(
                                        "Bank: {}\nOp: bank creation\nStatus: ok\nResult: {}\n\n",
                                        context.current_bank + 1,
                                        context.current_bank + 1
                                    )
                                    .as_bytes(),
                                )?;
                            }
                            Command::ChangeBank { id } => {
                                if id < 1 || id > context.banks.len() as u64 {
                                    writer.write_all(
                                    format!(
                                        "Bank: {}\nOp: bank changing\nStatus: error\nType: bank\nError: invalid bank id\n\n",
                                        context.current_bank + 1,
                                    )
                                    .as_bytes(),
                                )?;
                                } else {
                                    let new_current_bank = (id - 1) as usize;
                                    writer.write_all(
                                    format!("Bank: {}\nOp: bank changing\nStatus: ok\nResult: {}\n\n", 
                                        context.current_bank + 1,
                                        new_current_bank + 1,
                                    ).as_bytes(),
                                )?;
                                    context.current_bank = new_current_bank;
                                }
                            }
                            Command::RestoreBank { id } => {
                                if id < 1 || id > context.banks.len() as u64 {
                                    writer.write_all(
                                    format!(
                                        "Bank: {}\nOp: bank changing\nStatus: error\nType: bank\nError: invalid bank id\n\n",
                                        context.current_bank + 1,
                                    )
                                    .as_bytes(),
                                )?;
                                } else {
                                    let src_bank = &mut context.banks[context.current_bank];
                                    match Bank::restore(src_bank.get_all_operations()) {
                                        Ok(new_bank) => {
                                            writer.write_all(
                                            format!(
                                                "Bank: {}\nOp: new bank restored\nStatus: ok\nResult: {}\n\n",
                                                context.current_bank + 1,
                                                context.current_bank + 2,
                                            )
                                            .as_bytes(),
                                        )?;
                                            context.banks.push(new_bank);
                                        }
                                        Err(e) => {
                                            writer.write_all(
                                            format!(
                                                "Bank: {}\nOp: new bank restored\nStatus: error\nType: bank\nError: {}\n\n",
                                                context.current_bank + 1,
                                                e,
                                            )
                                            .as_bytes(),
                                        )?;
                                        }
                                    }
                                }
                            }
                            Command::RegisterAccount { balance } => {
                                let bank = &mut context.banks[context.current_bank];
                                let account = Account::new(balance);
                                match bank.register_account(account) {
                                    Ok(opperation_id) => {
                                        writer.write_all(
                                            format!(
                                                "Bank: {}\nOp: account creation\nOpID: {}\nStatus: ok\nResult: {}\n\n",
                                                context.current_bank + 1, opperation_id, account.id
                                            )
                                            .as_bytes(),
                                        )?;
                                    }
                                    Err(e) => {
                                        writer.write_all(
                                            format!(
                                                "Bank: {}\nOp: account creation\nStatus: error\nType: bank\nError: {}\n\n",
                                                context.current_bank, e
                                            ).as_bytes()
                                        )?;
                                    }
                                }
                            }
                            Command::GetBalance { id } => {
                                let bank = &context.banks[context.current_bank];
                                match bank.get_balance(id) {
                                    Ok(balance) => {
                                        writer.write_all(
                                            format!(
                                                "Bank: {}\nOp: balance info requested\nStatus: ok\nResult: {}\n\n",
                                                context.current_bank + 1, balance
                                            )
                                            .as_bytes(),
                                        )?;
                                    }
                                    Err(e) => {
                                        writer.write_all(
                                            format!(
                                                "Bank: {}\nOp: balance info requested\nStatus: fail\nResult: {}\n\n",
                                                context.current_bank + 1, e
                                            )
                                            .as_bytes(),
                                        )?;
                                    }
                                }
                            }
                            Command::Deposit { id, balance } => {
                                let bank = &mut context.banks[context.current_bank];
                                match bank.deposit(id, balance) {
                                    Ok(opperation_id) => {
                                        writer.write_all(
                                            format!(
                                                "Bank: {}\nOp: deposit\nOpID: {}\nStatus: ok\n\n",
                                                context.current_bank + 1,
                                                opperation_id,
                                            )
                                            .as_bytes(),
                                        )?;
                                    }
                                    Err(e) => {
                                        writer.write_all(
                                            format!(
                                            "Bank: {}\nOp: deposit\nStatus: fail\nResult: {}\n\n",
                                            context.current_bank + 1,
                                            e
                                        )
                                            .as_bytes(),
                                        )?;
                                    }
                                }
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
                    Err(e) => {
                        writer.write_all(
                            format!(
                                "Command: {}\nStatus: error\nType: parse\nError: {}\n\n",
                                line.trim(),
                                e
                            )
                            .as_bytes(),
                        )?;
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
