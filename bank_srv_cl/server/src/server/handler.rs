use crate::bank::account::{Account, AccountID};
use crate::bank::log::Operation;
use crate::bank::Bank;
use crate::server::command::{parse_command, Command, ParseError};
use std::io::{BufRead, Write};

pub struct Context {
    pub banks: Vec<Bank>,
    pub current_bank: usize,
}

type Result<T> = std::result::Result<T, Box<dyn std::error::Error>>;

fn handle_new_bank<W: Write>(context: &mut Context, writer: &mut W) -> Result<()> {
    context.banks.push(Bank::default());
    context.current_bank = context.banks.len() - 1;
    writer.write_all(
        format!(
            "Bank: {}\nOp: bank creation\nStatus: ok\nResult: {}\n\n",
            context.current_bank + 1,
            context.current_bank + 1
        )
        .as_bytes(),
    )?;

    Ok(())
}

fn handle_change_bank<W: Write>(id: u64, context: &mut Context, writer: &mut W) -> Result<()> {
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
            format!(
                "Bank: {}\nOp: bank changing\nStatus: ok\nResult: {}\n\n",
                context.current_bank + 1,
                new_current_bank + 1,
            )
            .as_bytes(),
        )?;
        context.current_bank = new_current_bank;
    }

    Ok(())
}

fn handle_restore_bank<W: Write>(id: u64, context: &mut Context, writer: &mut W) -> Result<()> {
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

    Ok(())
}

fn handle_register_account<W: Write>(
    balance: u64,
    context: &mut Context,
    writer: &mut W,
) -> Result<()> {
    let bank = &mut context.banks[context.current_bank];
    let account = Account::new(balance);
    match bank.register_account(account) {
        Ok(opperation_id) => {
            writer.write_all(
                format!(
                    "Bank: {}\nOp: account creation\nOpID: {}\nStatus: ok\nResult: {}\n\n",
                    context.current_bank + 1,
                    opperation_id,
                    account.id
                )
                .as_bytes(),
            )?;
        }
        Err(e) => {
            writer.write_all(
                format!(
                    "Bank: {}\nOp: account creation\nStatus: error\nType: bank\nError: {}\n\n",
                    context.current_bank, e
                )
                .as_bytes(),
            )?;
        }
    }

    Ok(())
}

fn handle_get_balance<W: Write>(
    id: AccountID,
    context: &mut Context,
    writer: &mut W,
) -> Result<()> {
    let bank = &context.banks[context.current_bank];
    match bank.get_balance(id) {
        Ok(balance) => {
            writer.write_all(
                format!(
                    "Bank: {}\nOp: balance info requested\nStatus: ok\nResult: {}\n\n",
                    context.current_bank + 1,
                    balance
                )
                .as_bytes(),
            )?;
        }
        Err(e) => {
            writer.write_all(
                format!(
                    "Bank: {}\nOp: balance info requested\nStatus: fail\nResult: {}\n\n",
                    context.current_bank + 1,
                    e
                )
                .as_bytes(),
            )?;
        }
    }

    Ok(())
}

fn handle_deposit<W: Write>(
    id: AccountID,
    amount: u64,
    context: &mut Context,
    writer: &mut W,
) -> Result<()> {
    let bank = &mut context.banks[context.current_bank];
    match bank.deposit(id, amount) {
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

    Ok(())
}

fn handle_withdraw<W: Write>(
    id: AccountID,
    amount: u64,
    context: &mut Context,
    writer: &mut W,
) -> Result<()> {
    let bank = &mut context.banks[context.current_bank];
    match bank.withdraw(id, amount) {
        Ok(opperation_id) => {
            writer.write_all(
                format!(
                    "Bank: {}\nOp: withdraw\nOpID: {}\nStatus: ok\n\n",
                    context.current_bank + 1,
                    opperation_id,
                )
                .as_bytes(),
            )?;
        }
        Err(e) => {
            writer.write_all(
                format!(
                    "Bank: {}\nOp: withdraw\nStatus: error\nType: bank\nError: {}\n\n",
                    context.current_bank + 1,
                    e
                )
                .as_bytes(),
            )?;
        }
    }

    Ok(())
}

fn handle_transfer<W: Write>(
    sender: AccountID,
    reciever: AccountID,
    amount: u64,
    context: &mut Context,
    writer: &mut W,
) -> Result<()> {
    let bank = &mut context.banks[context.current_bank];
    match bank.transfer(sender, reciever, amount) {
        Ok(opperation_id) => {
            writer.write_all(
                format!(
                    "Bank: {}\nOp: transfer\nOpID: {}\nStatus: ok\n\n",
                    context.current_bank + 1,
                    opperation_id,
                )
                .as_bytes(),
            )?;
        }
        Err(e) => {
            writer.write_all(
                format!(
                    "Bank: {}\nOp: transfer\nStatus: error\nType: bank\nError: {}\n\n",
                    context.current_bank + 1,
                    e
                )
                .as_bytes(),
            )?;
        }
    }

    Ok(())
}

fn operations_as_string<'a, I: Iterator<Item = &'a Operation>>(operations: I) -> String {
    let operations: Vec<String> = operations.map(|op| op.to_string()).collect();
    operations.join("\n")
}

fn handle_list_account_operations<W: Write>(
    id: AccountID,
    context: &mut Context,
    writer: &mut W,
) -> Result<()> {
    let bank = &context.banks[context.current_bank];
    let operations = bank.get_account_operations(id);

    writer.write_all(
        format!(
            "Bank: {}\nOp: account operations list requested\nStatus: ok\nResult: \n{}\n\n",
            context.current_bank + 1,
            operations_as_string(operations),
        )
        .as_bytes(),
    )?;

    Ok(())
}

fn handle_list_all_operations<W: Write>(context: &mut Context, writer: &mut W) -> Result<()> {
    let bank = &context.banks[context.current_bank];
    let operations = bank.get_all_operations();

    writer.write_all(
        format!(
            "Bank: {}\nOp: all operations list requested\nStatus: ok\nResult: \n{}\n\n",
            context.current_bank + 1,
            operations_as_string(operations),
        )
        .as_bytes(),
    )?;

    Ok(())
}

fn handle_quit<W: Write>(writer: &mut W) -> Result<()> {
    writer.write_all("Bye-byte\n\n".as_bytes())?;

    Ok(())
}

fn handle_command(command: &Command, context: &mut Context, writer: &mut impl Write) -> Result<()> {
    match *command {
        Command::NewBank => handle_new_bank(context, writer)?,
        Command::ChangeBank { id } => handle_change_bank(id, context, writer)?,
        Command::RestoreBank { id } => handle_restore_bank(id, context, writer)?,
        Command::RegisterAccount { balance } => handle_register_account(balance, context, writer)?,
        Command::GetBalance { id } => handle_get_balance(id, context, writer)?,
        Command::Deposit { id, balance } => handle_deposit(id, balance, context, writer)?,
        Command::Withdraw { id, balance } => handle_withdraw(id, balance, context, writer)?,
        Command::Transfer {
            sender,
            reciever,
            amount,
        } => handle_transfer(sender, reciever, amount, context, writer)?,

        Command::ListAccountOperations { id } => {
            handle_list_account_operations(id, context, writer)?
        }
        Command::ListAllOperations => handle_list_all_operations(context, writer)?,
        Command::Quit => handle_quit(writer)?,
    };

    Ok(())
}

fn handle_parse_error(e: ParseError, command: &str, writer: &mut impl Write) -> Result<()> {
    writer.write_all(
        format!(
            "Command: {}\nStatus: error\nType: parse\nError: {}\n\n",
            command.trim(),
            e
        )
        .as_bytes(),
    )?;

    Ok(())
}

pub fn handle<R: BufRead, W: Write>(
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
                        handle_command(&command, context, writer)?;
                        if command == Command::Quit {
                            println!("Client quited");
                            break;
                        }
                    }
                    Err(e) => handle_parse_error(e, &line, writer)?,
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
