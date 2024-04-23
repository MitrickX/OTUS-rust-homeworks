use crate::bank::account::{Account, AccountID};
use crate::bank::log::Operation;
use crate::bank::Bank;
use crate::server::command::{parse_command, Command, ParseError};
use std::io::{BufRead, Write};

#[derive(Default, Clone, Debug)]
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
            "Bank: {}\nStatus: ok\nResult: {}\n\n",
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
                "Bank: {}\nStatus: error\nType: bank\nError: invalid bank id\n\n",
                context.current_bank + 1,
            )
            .as_bytes(),
        )?;
    } else {
        let new_current_bank = (id - 1) as usize;
        writer.write_all(
            format!(
                "Bank: {}\nStatus: ok\nResult: {}\n\n",
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
                "Bank: {}\nStatus: error\nType: bank\nError: invalid bank id\n\n",
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
                        "Bank: {}\nStatus: ok\nResult: {}\n\n",
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
                        "Bank: {}\nStatus: error\nType: bank\nError: {}\n\n",
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
    if context.banks.is_empty() {
        context.banks.push(Bank::default());
    }
    let bank = &mut context.banks[context.current_bank];
    let account = Account::new(balance);
    match bank.register_account(account) {
        Ok(opperation_id) => {
            writer.write_all(
                format!(
                    "Bank: {}\nOpID: {}\nStatus: ok\nResult: {}\n\n",
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
                    "Bank: {}\nStatus: error\nType: bank\nError: {}\n\n",
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
                    "Bank: {}\nStatus: ok\nResult: {}\n\n",
                    context.current_bank + 1,
                    balance
                )
                .as_bytes(),
            )?;
        }
        Err(e) => {
            writer.write_all(
                format!(
                    "Bank: {}\nStatus: fail\nResult: {}\n\n",
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
                    "Bank: {}\nOpID: {}\nStatus: ok\n\n",
                    context.current_bank + 1,
                    opperation_id,
                )
                .as_bytes(),
            )?;
        }
        Err(e) => {
            writer.write_all(
                format!(
                    "Bank: {}\nStatus: fail\nResult: {}\n\n",
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
                    "Bank: {}\nOpID: {}\nStatus: ok\n\n",
                    context.current_bank + 1,
                    opperation_id,
                )
                .as_bytes(),
            )?;
        }
        Err(e) => {
            writer.write_all(
                format!(
                    "Bank: {}\nStatus: error\nType: bank\nError: {}\n\n",
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
                    "Bank: {}\nOpID: {}\nStatus: ok\n\n",
                    context.current_bank + 1,
                    opperation_id,
                )
                .as_bytes(),
            )?;
        }
        Err(e) => {
            writer.write_all(
                format!(
                    "Bank: {}\nStatus: error\nType: bank\nError: {}\n\n",
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
            "Bank: {}\nStatus: ok\nResult: \n{}\n\n",
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
            "Bank: {}\nStatus: ok\nResult: \n{}\n\n",
            context.current_bank + 1,
            operations_as_string(operations),
        )
        .as_bytes(),
    )?;

    Ok(())
}

fn handle_quit<W: Write>(writer: &mut W) -> Result<()> {
    writer.write_all("Bye bye\n\n".as_bytes())?;

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

pub fn handle<R: BufRead, W: Write, T: Write>(
    context: &mut Context,
    reader: &mut R,
    writer: &mut W,
    terminal: &mut T,
) -> Result<()> {
    loop {
        let mut line = String::new();
        match reader.read_line(&mut line) {
            Ok(n) => {
                if n == 0 {
                    terminal.write_all("Client disconnected\n".as_bytes())?;
                    break;
                }

                match parse_command(&line) {
                    Ok(command) => {
                        handle_command(&command, context, writer)?;
                        if command == Command::Quit {
                            terminal.write_all("Client quited\n".as_bytes())?;
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

#[cfg(test)]
mod tests {
    use super::*;
    use crate::bank::log::OperationKind;
    use std::str::from_utf8;

    #[test]
    fn unknown_command_works() {
        let mut context = Context::default();
        let mut reader = "test_command".as_bytes();
        let mut writer = Vec::new();
        let mut terminal = Vec::new();

        handle(&mut context, &mut reader, &mut writer, &mut terminal).unwrap();

        assert_eq!(
            from_utf8(writer.as_slice()).unwrap(),
            "Command: test_command\nStatus: error\nType: parse\nError: unknown command\n\n"
                .to_owned()
        );
    }

    #[test]
    fn handle_empty_command_works() {
        let mut context = Context::default();
        let mut reader = "".as_bytes();
        let mut writer = Vec::new();
        let mut terminal = Vec::new();

        handle(&mut context, &mut reader, &mut writer, &mut terminal).unwrap();

        assert_eq!(
            from_utf8(terminal.as_slice()).unwrap(),
            "Client disconnected\n".to_owned()
        );
    }

    #[test]
    fn handle_quit_command_works() {
        let mut context = Context::default();
        let mut reader = "quit".as_bytes();
        let mut writer = Vec::new();
        let mut terminal = Vec::new();

        handle(&mut context, &mut reader, &mut writer, &mut terminal).unwrap();

        assert_eq!(
            from_utf8(writer.as_slice()).unwrap(),
            "Bye bye\n\n".to_owned()
        );
        assert_eq!(
            from_utf8(terminal.as_slice()).unwrap(),
            "Client quited\n".to_owned()
        );
    }

    #[test]
    fn handle_new_bank_command() {
        let mut context = Context::default();
        let mut reader = "new_bank".as_bytes();
        let mut writer = Vec::new();
        let mut terminal = Vec::new();

        handle(&mut context, &mut reader, &mut writer, &mut terminal).unwrap();

        assert_eq!(
            from_utf8(writer.as_slice()).unwrap(),
            "Bank: 1\nStatus: ok\nResult: 1\n\n".to_owned(),
        );
    }

    #[test]
    fn handle_register_account_works() {
        let mut context = Context::default();

        let input = vec!["register_account", "register_account 100"].join("\n");

        let mut reader = input.as_bytes();

        let mut writer = Vec::new();
        let mut terminal = Vec::new();

        handle(&mut context, &mut reader, &mut writer, &mut terminal).unwrap();

        let operations: Vec<&Operation> = context.banks[context.current_bank]
            .get_all_operations()
            .collect();

        let operation = operations[0];
        let operation_id = operation.id;

        let account_id = if let OperationKind::Register { id, .. } = operation.kind {
            id
        } else {
            AccountID::new()
        };

        let expected = vec![
            format!(
                "Command: register_account\nStatus: error\nType: parse\nError: {}\n\n",
                ParseError::RequireArguments(vec!["balance".to_string()]),
            ),
            format!(
                "Bank: 1\nOpID: {}\nStatus: ok\nResult: {}\n\n",
                operation_id, account_id,
            ),
        ]
        .join("");

        assert_eq!(from_utf8(writer.as_slice()).unwrap(), expected);
    }

    #[test]
    fn handle_get_balance_works() {
        let mut reader = "register_account 100".as_bytes();

        let mut writer = Vec::new();
        let mut terminal = Vec::new();

        let mut context = Context::default();
        handle(&mut context, &mut reader, &mut writer, &mut terminal).unwrap();

        let operations: Vec<&Operation> = context.banks[context.current_bank]
            .get_all_operations()
            .collect();

        let account_id = if let OperationKind::Register { id, .. } = operations[0].kind {
            id
        } else {
            AccountID::new()
        };

        let input = vec![
            "get_balance".to_owned(),
            "get_balance test".to_owned(),
            format!("get_balance {}", account_id.to_string()),
        ]
        .join("\n");

        let mut reader = input.as_bytes();

        let mut context = context.clone();
        handle(&mut context, &mut reader, &mut writer, &mut terminal).unwrap();

        let expected = vec![
            format!(
                "Bank: 1\nOpID: {}\nStatus: ok\nResult: {}\n\n",
                operations[0].id, account_id,
            ),
            format!(
                "Command: get_balance\nStatus: error\nType: parse\nError: {}\n\n",
                ParseError::RequireArguments(vec!["account_id".to_string()]),
            ),
            format!(
                "Command: get_balance test\nStatus: error\nType: parse\nError: {}\n\n",
                ParseError::InvalidArgumentAccountID(
                    "account_id".to_owned(),
                    AccountID::parse_str("test").unwrap_err()
                )
            ),
            format!("Bank: 1\nStatus: ok\nResult: {}\n\n", 100),
        ]
        .join("");

        assert_eq!(from_utf8(writer.as_slice()).unwrap(), expected);
    }

    #[test]
    fn handle_deposit_works() {
        let mut reader = "register_account 100".as_bytes();

        let mut writer = Vec::new();
        let mut terminal = Vec::new();

        let mut context = Context::default();
        handle(&mut context, &mut reader, &mut writer, &mut terminal).unwrap();

        let operations: Vec<&Operation> = context.banks[context.current_bank]
            .get_all_operations()
            .collect();

        let account_id = if let OperationKind::Register { id, .. } = operations[0].kind {
            id
        } else {
            AccountID::new()
        };

        let input = vec![
            "deposit".to_owned(),
            "deposit test 10".to_owned(),
            format!("deposit {} test", account_id.to_string()),
            format!("deposit {} 100", account_id.to_string()),
        ]
        .join("\n");

        let mut reader = input.as_bytes();

        let mut context = context.clone();
        handle(&mut context, &mut reader, &mut writer, &mut terminal).unwrap();

        let operations: Vec<&Operation> = context.banks[context.current_bank]
            .get_all_operations()
            .collect();

        let expected = vec![
            format!(
                "Bank: 1\nOpID: {}\nStatus: ok\nResult: {}\n\n",
                operations[0].id, account_id,
            ),
            format!(
                "Command: deposit\nStatus: error\nType: parse\nError: {}\n\n",
                ParseError::RequireArguments(vec!["account_id".to_owned(), "amount".to_owned()]),
            ),
            format!(
                "Command: deposit test 10\nStatus: error\nType: parse\nError: {}\n\n",
                ParseError::InvalidArgumentAccountID(
                    "account_id".to_owned(),
                    AccountID::parse_str("test").unwrap_err()
                )
            ),
            format!(
                "Command: deposit {} test\nStatus: error\nType: parse\nError: {}\n\n",
                account_id.to_string(),
                ParseError::InvalidArgumentUint(
                    "amount".to_owned(),
                    "test".parse::<u64>().unwrap_err(),
                )
            ),
            format!("Bank: 1\nOpID: {}\nStatus: ok\n\n", operations[1].id),
        ]
        .join("");

        assert_eq!(from_utf8(writer.as_slice()).unwrap(), expected);
    }

    #[test]
    fn handle_withdraw_works() {
        let mut reader = "register_account 100".as_bytes();

        let mut writer = Vec::new();
        let mut terminal = Vec::new();

        let mut context = Context::default();
        handle(&mut context, &mut reader, &mut writer, &mut terminal).unwrap();

        let operations: Vec<&Operation> = context.banks[context.current_bank]
            .get_all_operations()
            .collect();

        let account_id = if let OperationKind::Register { id, .. } = operations[0].kind {
            id
        } else {
            AccountID::new()
        };

        let input = vec![
            "withdraw".to_owned(),
            "withdraw test 10".to_owned(),
            format!("withdraw {} test", account_id.to_string()),
            format!("withdraw {} 100", account_id.to_string()),
            format!("withdraw {} 100", account_id.to_string()),
        ]
        .join("\n");

        let mut reader = input.as_bytes();

        let mut context = context.clone();
        handle(&mut context, &mut reader, &mut writer, &mut terminal).unwrap();

        let operations: Vec<&Operation> = context.banks[context.current_bank]
            .get_all_operations()
            .collect();

        let expected = vec![
            format!(
                "Bank: 1\nOpID: {}\nStatus: ok\nResult: {}\n\n",
                operations[0].id, account_id,
            ),
            format!(
                "Command: withdraw\nStatus: error\nType: parse\nError: {}\n\n",
                ParseError::RequireArguments(vec!["account_id".to_owned(), "amount".to_owned()]),
            ),
            format!(
                "Command: withdraw test 10\nStatus: error\nType: parse\nError: {}\n\n",
                ParseError::InvalidArgumentAccountID(
                    "account_id".to_owned(),
                    AccountID::parse_str("test").unwrap_err()
                )
            ),
            format!(
                "Command: withdraw {} test\nStatus: error\nType: parse\nError: {}\n\n",
                account_id.to_string(),
                ParseError::InvalidArgumentUint(
                    "amount".to_owned(),
                    "test".parse::<u64>().unwrap_err(),
                )
            ),
            format!("Bank: 1\nOpID: {}\nStatus: ok\n\n", operations[1].id),
            "Bank: 1\nStatus: error\nType: bank\nError: Insufficient funds\n\n".to_owned(),
        ]
        .join("");

        assert_eq!(from_utf8(writer.as_slice()).unwrap(), expected);
    }

    #[test]
    fn handle_transfer_works() {
        let input = vec![
            "register_account 100".to_owned(),
            "register_account 50".to_owned(),
        ]
        .join("\n");

        let mut reader = input.as_bytes();

        let mut writer = Vec::new();
        let mut terminal = Vec::new();

        let mut context = Context::default();
        handle(&mut context, &mut reader, &mut writer, &mut terminal).unwrap();

        let operations: Vec<&Operation> = context.banks[context.current_bank]
            .get_all_operations()
            .collect();

        let account1_id = if let OperationKind::Register { id, .. } = operations[0].kind {
            id
        } else {
            AccountID::new()
        };

        let account2_id = if let OperationKind::Register { id, .. } = operations[1].kind {
            id
        } else {
            AccountID::new()
        };

        let input = vec![
            "transfer".to_owned(),
            "transfer test1 test2 50".to_owned(),
            format!("transfer {} test2 50", account1_id.to_string()),
            format!("transfer test1 {} 50", account2_id.to_string()),
            format!(
                "transfer {} {} test",
                account1_id.to_string(),
                account2_id.to_string()
            ),
            format!(
                "transfer {} {} 50",
                account1_id.to_string(),
                account2_id.to_string()
            ),
            format!(
                "transfer {} {} 500",
                account2_id.to_string(),
                account1_id.to_string()
            ),
        ]
        .join("\n");

        let mut reader = input.as_bytes();

        let mut context = context.clone();
        handle(&mut context, &mut reader, &mut writer, &mut terminal).unwrap();

        let operations: Vec<&Operation> = context.banks[context.current_bank]
            .get_all_operations()
            .collect();

        let expected = vec![
            format!(
                "Bank: 1\nOpID: {}\nStatus: ok\nResult: {}\n\n",
                operations[0].id, account1_id,
            ),
            format!(
                "Bank: 1\nOpID: {}\nStatus: ok\nResult: {}\n\n",
                operations[1].id, account2_id,
            ),
            format!(
                "Command: transfer\nStatus: error\nType: parse\nError: {}\n\n",
                ParseError::RequireArguments(vec![
                    "sender_account_id".to_owned(),
                    "reciever_account_id".to_owned(),
                    "amount".to_owned()
                ]),
            ),
            format!(
                "Command: transfer test1 test2 50\nStatus: error\nType: parse\nError: {}\n\n",
                ParseError::InvalidArgumentAccountID(
                    "sender_account_id".to_owned(),
                    AccountID::parse_str("test1").unwrap_err()
                )
            ),
            format!(
                "Command: transfer {} test2 50\nStatus: error\nType: parse\nError: {}\n\n",
                account1_id.to_string(),
                ParseError::InvalidArgumentAccountID(
                    "reciever_account_id".to_owned(),
                    AccountID::parse_str("test2").unwrap_err()
                )
            ),
            format!(
                "Command: transfer test1 {} 50\nStatus: error\nType: parse\nError: {}\n\n",
                account2_id.to_string(),
                ParseError::InvalidArgumentAccountID(
                    "sender_account_id".to_owned(),
                    AccountID::parse_str("test1").unwrap_err()
                )
            ),
            format!(
                "Command: transfer {} {} test\nStatus: error\nType: parse\nError: invalid argument amount: {}\n\n",
                account1_id.to_string(),
                account2_id.to_string(),
                "test".parse::<u64>().unwrap_err(),
            ),
            format!("Bank: 1\nOpID: {}\nStatus: ok\n\n", operations[2].id),
            "Bank: 1\nStatus: error\nType: bank\nError: Insufficient funds\n\n".to_owned(),
        ]
        .join("");

        assert_eq!(from_utf8(writer.as_slice()).unwrap(), expected);
    }
}
