use crate::bank::account::{Account, AccountID};
use crate::bank::log::Operation;
use crate::bank::Bank;
use crate::server::command::{parse_command, Command, ParseError};
use std::io::{BufRead, Write};
use std::sync::{Arc, RwLock};

#[derive(Default, Clone, Debug)]
pub struct Context {
    pub banks: Vec<Bank>,
    pub current_bank: usize,
}

type ARWLockContext = Arc<RwLock<Context>>;

type Result<T> = std::result::Result<T, Box<dyn std::error::Error>>;

fn handle_new_bank<W: Write>(lock_context: ARWLockContext, writer: &mut W) -> Result<()> {
    let context = lock_context.read().unwrap();
    let prev_bank_id = if context.banks.is_empty() {
        0
    } else {
        context.current_bank + 1
    };

    drop(context);

    let mut context = lock_context.write().unwrap();
    context.banks.push(Bank::default());
    context.current_bank = context.banks.len() - 1;

    let current_bank_id = context.current_bank + 1;

    writer.write_all(
        format!(
            "Bank: {}\nStatus: ok\nResult: {}\n\n",
            prev_bank_id, current_bank_id
        )
        .as_bytes(),
    )?;

    Ok(())
}

fn handle_change_bank<W: Write>(
    id: u64,
    lock_context: ARWLockContext,
    writer: &mut W,
) -> Result<()> {
    let context = lock_context.read().unwrap();
    if id < 1 || id > context.banks.len() as u64 {
        writer.write_all(
            format!(
                "Bank: {}\nStatus: error\nType: bank\nError: invalid bank id\n\n",
                context.current_bank + 1,
            )
            .as_bytes(),
        )?;

        return Ok(());
    }

    drop(context);

    let mut context = lock_context.write().unwrap();
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

    Ok(())
}

fn handle_which_bank<W: Write>(lock_context: ARWLockContext, writer: &mut W) -> Result<()> {
    let mut context = lock_context.write().unwrap();
    if context.banks.is_empty() {
        context.banks.push(Bank::default());
    }

    let current_bank = context.current_bank + 1;
    writer.write_all(
        format!(
            "Bank: {}\nStatus: ok\nResult: {}\n\n",
            current_bank, current_bank
        )
        .as_bytes(),
    )?;

    Ok(())
}

fn handle_restore_bank<W: Write>(
    id: u64,
    lock_context: ARWLockContext,
    writer: &mut W,
) -> Result<()> {
    let context = lock_context.read().unwrap();

    if id < 1 || id > context.banks.len() as u64 {
        writer.write_all(
            format!(
                "Bank: {}\nStatus: error\nType: bank\nError: invalid bank id\n\n",
                context.current_bank + 1,
            )
            .as_bytes(),
        )?;

        return Ok(());
    }

    drop(context);

    let mut context = lock_context.write().unwrap();
    let current_bank = context.current_bank;

    let src_bank = &mut context.banks[current_bank];
    match Bank::restore(src_bank.get_all_operations()) {
        Ok(new_bank) => {
            writer.write_all(
                format!(
                    "Bank: {}\nStatus: ok\nResult: {}\n\n",
                    current_bank + 1,
                    current_bank + 2,
                )
                .as_bytes(),
            )?;
            context.banks.push(new_bank);
            context.current_bank = context.banks.len() - 1;
        }
        Err(e) => {
            writer.write_all(
                format!(
                    "Bank: {}\nStatus: error\nType: bank\nError: {}\n\n",
                    current_bank + 1,
                    e,
                )
                .as_bytes(),
            )?;
        }
    }

    Ok(())
}

fn handle_register_account<W: Write>(
    balance: u64,
    lock_context: ARWLockContext,
    writer: &mut W,
) -> Result<()> {
    let mut context = lock_context.write().unwrap();
    if context.banks.is_empty() {
        context.banks.push(Bank::default());
    }

    let current_bank = context.current_bank;
    let bank = &mut context.banks[current_bank];
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
    lock_context: ARWLockContext,
    writer: &mut W,
) -> Result<()> {
    let context = lock_context.read().unwrap();
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
    lock_context: ARWLockContext,
    writer: &mut W,
) -> Result<()> {
    let mut context = lock_context.write().unwrap();
    let current_bank = context.current_bank;
    let bank = &mut context.banks[current_bank];
    match bank.deposit(id, amount) {
        Ok(opperation_id) => {
            writer.write_all(
                format!(
                    "Bank: {}\nOpID: {}\nStatus: ok\n\n",
                    current_bank + 1,
                    opperation_id,
                )
                .as_bytes(),
            )?;
        }
        Err(e) => {
            writer.write_all(
                format!(
                    "Bank: {}\nStatus: fail\nResult: {}\n\n",
                    current_bank + 1,
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
    lock_context: ARWLockContext,
    writer: &mut W,
) -> Result<()> {
    let mut context = lock_context.write().unwrap();
    let current_bank = context.current_bank;
    let bank = &mut context.banks[current_bank];
    match bank.withdraw(id, amount) {
        Ok(opperation_id) => {
            writer.write_all(
                format!(
                    "Bank: {}\nOpID: {}\nStatus: ok\n\n",
                    current_bank + 1,
                    opperation_id,
                )
                .as_bytes(),
            )?;
        }
        Err(e) => {
            writer.write_all(
                format!(
                    "Bank: {}\nStatus: error\nType: bank\nError: {}\n\n",
                    current_bank + 1,
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
    lock_context: ARWLockContext,
    writer: &mut W,
) -> Result<()> {
    let mut context = lock_context.write().unwrap();
    let current_bank = context.current_bank;
    let bank = &mut context.banks[current_bank];
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
    lock_context: ARWLockContext,
    writer: &mut W,
) -> Result<()> {
    let context = lock_context.read().unwrap();
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

fn handle_list_all_operations<W: Write>(
    lock_context: ARWLockContext,
    writer: &mut W,
) -> Result<()> {
    let context = lock_context.read().unwrap();
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

fn handle_command(
    command: &Command,
    lock_context: ARWLockContext,
    writer: &mut impl Write,
) -> Result<()> {
    match *command {
        Command::NewBank => handle_new_bank(lock_context, writer)?,
        Command::ChangeBank { id } => handle_change_bank(id, lock_context, writer)?,
        Command::RestoreBank { id } => handle_restore_bank(id, lock_context, writer)?,
        Command::WhichBank => handle_which_bank(lock_context, writer)?,
        Command::RegisterAccount { balance } => {
            handle_register_account(balance, lock_context, writer)?
        }
        Command::GetBalance { id } => handle_get_balance(id, lock_context, writer)?,
        Command::Deposit { id, balance } => handle_deposit(id, balance, lock_context, writer)?,
        Command::Withdraw { id, balance } => handle_withdraw(id, balance, lock_context, writer)?,
        Command::Transfer {
            sender,
            reciever,
            amount,
        } => handle_transfer(sender, reciever, amount, lock_context, writer)?,

        Command::ListAccountOperations { id } => {
            handle_list_account_operations(id, lock_context, writer)?
        }
        Command::ListAllOperations => handle_list_all_operations(lock_context, writer)?,
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
    lock_context: ARWLockContext,
    reader: &mut R,
    writer: &mut W,
    terminal: &mut T,
) -> Result<()> {
    let original_lock_context = lock_context;
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
                        let lock_context = Arc::clone(&original_lock_context);
                        handle_command(&command, lock_context, writer)?;
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
        let original_lock_context = Arc::new(RwLock::new(Context::default()));

        let mut reader = "test_command".as_bytes();
        let mut writer = Vec::new();
        let mut terminal = Vec::new();

        let lock_context = Arc::clone(&original_lock_context);

        handle(
            Arc::clone(&lock_context),
            &mut reader,
            &mut writer,
            &mut terminal,
        )
        .unwrap();

        assert_eq!(
            from_utf8(writer.as_slice()).unwrap(),
            "Command: test_command\nStatus: error\nType: parse\nError: unknown command\n\n"
                .to_owned()
        );
    }

    #[test]
    fn handle_empty_command_works() {
        let original_lock_context = Arc::new(RwLock::new(Context::default()));

        let mut reader = "".as_bytes();
        let mut writer = Vec::new();
        let mut terminal = Vec::new();

        let lock_context = Arc::clone(&original_lock_context);
        handle(lock_context, &mut reader, &mut writer, &mut terminal).unwrap();

        assert_eq!(
            from_utf8(terminal.as_slice()).unwrap(),
            "Client disconnected\n".to_owned()
        );
    }

    #[test]
    fn handle_quit_command_works() {
        let original_lock_context = Arc::new(RwLock::new(Context::default()));

        let mut reader = "quit".as_bytes();
        let mut writer = Vec::new();
        let mut terminal = Vec::new();

        let lock_context = Arc::clone(&original_lock_context);
        handle(lock_context, &mut reader, &mut writer, &mut terminal).unwrap();

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
        let original_lock_context = Arc::new(RwLock::new(Context::default()));

        let mut reader = "new_bank".as_bytes();
        let mut writer = Vec::new();
        let mut terminal = Vec::new();

        let lock_context = Arc::clone(&original_lock_context);
        handle(lock_context, &mut reader, &mut writer, &mut terminal).unwrap();

        assert_eq!(
            from_utf8(writer.as_slice()).unwrap(),
            "Bank: 0\nStatus: ok\nResult: 1\n\n".to_owned(),
        );
    }

    #[test]
    fn handle_which_bank_command() {
        let original_lock_context = Arc::new(RwLock::new(Context::default()));

        let input = vec!["which_bank", "new_bank", "which_bank"].join("\n");
        let mut reader = input.as_bytes();

        let mut writer = Vec::new();
        let mut terminal = Vec::new();

        let lock_context = Arc::clone(&original_lock_context);
        handle(lock_context, &mut reader, &mut writer, &mut terminal).unwrap();

        let expected = vec![
            "Bank: 1\nStatus: ok\nResult: 1\n\n".to_owned(),
            "Bank: 1\nStatus: ok\nResult: 2\n\n".to_owned(),
            "Bank: 2\nStatus: ok\nResult: 2\n\n".to_owned(),
        ]
        .join("");

        assert_eq!(from_utf8(writer.as_slice()).unwrap(), expected);
    }

    #[test]
    fn handle_change_bank_command() {
        let original_lock_context = Arc::new(RwLock::new(Context::default()));

        let input = vec![
            "new_bank",
            "new_bank",
            "new_bank",
            "change_bank 2",
            "change_bank 3",
            "change_bank 1",
            "change_bank 4",
            "change_bank 100",
            "change_bank 0",
        ]
        .join("\n");
        let mut reader = input.as_bytes();
        let mut writer = Vec::new();
        let mut terminal = Vec::new();

        let lock_context = Arc::clone(&original_lock_context);
        handle(lock_context, &mut reader, &mut writer, &mut terminal).unwrap();
        let expected = vec![
            "Bank: 0\nStatus: ok\nResult: 1\n\n".to_owned(),
            "Bank: 1\nStatus: ok\nResult: 2\n\n".to_owned(),
            "Bank: 2\nStatus: ok\nResult: 3\n\n".to_owned(),
            "Bank: 3\nStatus: ok\nResult: 2\n\n".to_owned(),
            "Bank: 2\nStatus: ok\nResult: 3\n\n".to_owned(),
            "Bank: 3\nStatus: ok\nResult: 1\n\n".to_owned(),
            "Bank: 1\nStatus: error\nType: bank\nError: invalid bank id\n\n".to_owned(),
            "Bank: 1\nStatus: error\nType: bank\nError: invalid bank id\n\n".to_owned(),
            "Bank: 1\nStatus: error\nType: bank\nError: invalid bank id\n\n".to_owned(),
        ]
        .join("");

        assert_eq!(from_utf8(writer.as_slice()).unwrap(), expected);
    }

    #[test]
    fn handle_register_account_works() {
        let original_lock_context = Arc::new(RwLock::new(Context::default()));

        let input = vec!["register_account", "register_account 100"].join("\n");

        let mut reader = input.as_bytes();

        let mut writer = Vec::new();
        let mut terminal = Vec::new();

        let lock_context = Arc::clone(&original_lock_context);

        handle(lock_context, &mut reader, &mut writer, &mut terminal).unwrap();

        let lock_context = Arc::clone(&original_lock_context);
        let context = lock_context.read().unwrap();
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
                ParseError::RequireArguments {
                    args: vec!["balance".to_string()]
                },
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

        let original_lock_context = Arc::new(RwLock::new(Context::default()));
        let lock_context = Arc::clone(&original_lock_context);

        // TODO: hanging here, why?

        handle(lock_context, &mut reader, &mut writer, &mut terminal).unwrap();

        let lock_context = Arc::clone(&original_lock_context);
        let context = lock_context.read().unwrap();

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

        let lock_context = Arc::clone(&original_lock_context);
        handle(lock_context, &mut reader, &mut writer, &mut terminal).unwrap();

        let expected = vec![
            format!(
                "Bank: 1\nOpID: {}\nStatus: ok\nResult: {}\n\n",
                operations[0].id, account_id,
            ),
            format!(
                "Command: get_balance\nStatus: error\nType: parse\nError: {}\n\n",
                ParseError::RequireArguments {
                    args: vec!["account_id".to_string()]
                },
            ),
            format!(
                "Command: get_balance test\nStatus: error\nType: parse\nError: {}\n\n",
                ParseError::InvalidArgumentAccountID {
                    name: "account_id".to_owned(),
                    e: AccountID::parse_str("test").unwrap_err()
                }
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

        let original_lock_context = Arc::new(RwLock::new(Context::default()));
        let lock_context = Arc::clone(&original_lock_context);

        handle(lock_context, &mut reader, &mut writer, &mut terminal).unwrap();

        let account_id = {
            let lock_context = Arc::clone(&original_lock_context);
            let context = lock_context.read().unwrap();
            let current_bank = context.current_bank;
            let operations: Vec<&Operation> =
                context.banks[current_bank].get_all_operations().collect();

            if let OperationKind::Register { id, .. } = operations[0].kind {
                id
            } else {
                AccountID::new()
            }
        };

        let input = vec![
            "deposit".to_owned(),
            "deposit test 10".to_owned(),
            format!("deposit {} test", account_id.to_string()),
            format!("deposit {} 100", account_id.to_string()),
        ]
        .join("\n");

        let mut reader = input.as_bytes();

        let lock_context = Arc::clone(&original_lock_context);
        handle(lock_context, &mut reader, &mut writer, &mut terminal).unwrap();

        let expected = {
            let lock_context = Arc::clone(&original_lock_context);
            let context = lock_context.read().unwrap();
            let operations: Vec<&Operation> = context.banks[context.current_bank]
                .get_all_operations()
                .collect();

            vec![
                format!(
                    "Bank: 1\nOpID: {}\nStatus: ok\nResult: {}\n\n",
                    operations[0].id, account_id,
                ),
                format!(
                    "Command: deposit\nStatus: error\nType: parse\nError: {}\n\n",
                    ParseError::RequireArguments {
                        args: vec!["account_id".to_owned(), "amount".to_owned()]
                    },
                ),
                format!(
                    "Command: deposit test 10\nStatus: error\nType: parse\nError: {}\n\n",
                    ParseError::InvalidArgumentAccountID {
                        name: "account_id".to_owned(),
                        e: AccountID::parse_str("test").unwrap_err()
                    }
                ),
                format!(
                    "Command: deposit {} test\nStatus: error\nType: parse\nError: {}\n\n",
                    account_id.to_string(),
                    ParseError::InvalidArgumentUint {
                        name: "amount".to_owned(),
                        e: "test".parse::<u64>().unwrap_err(),
                    }
                ),
                format!("Bank: 1\nOpID: {}\nStatus: ok\n\n", operations[1].id),
            ]
            .join("")
        };

        assert_eq!(from_utf8(writer.as_slice()).unwrap(), expected);
    }

    #[test]
    fn handle_withdraw_works() {
        let mut reader = "register_account 100".as_bytes();

        let mut writer = Vec::new();
        let mut terminal = Vec::new();

        let original_lock_context = Arc::new(RwLock::new(Context::default()));
        let lock_context = Arc::clone(&original_lock_context);

        handle(lock_context, &mut reader, &mut writer, &mut terminal).unwrap();

        let account_id = {
            let lock_context = Arc::clone(&original_lock_context);
            let context = lock_context.read().unwrap();

            let operations: Vec<&Operation> = context.banks[context.current_bank]
                .get_all_operations()
                .collect();

            if let OperationKind::Register { id, .. } = operations[0].kind {
                id
            } else {
                AccountID::new()
            }
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

        let lock_context = Arc::clone(&original_lock_context);
        handle(lock_context, &mut reader, &mut writer, &mut terminal).unwrap();

        let expected = {
            let lock_context = Arc::clone(&original_lock_context);
            let context = lock_context.read().unwrap();
            let operations: Vec<&Operation> = context.banks[context.current_bank]
                .get_all_operations()
                .collect();

            vec![
                format!(
                    "Bank: 1\nOpID: {}\nStatus: ok\nResult: {}\n\n",
                    operations[0].id, account_id,
                ),
                format!(
                    "Command: withdraw\nStatus: error\nType: parse\nError: {}\n\n",
                    ParseError::RequireArguments {
                        args: vec!["account_id".to_owned(), "amount".to_owned()]
                    },
                ),
                format!(
                    "Command: withdraw test 10\nStatus: error\nType: parse\nError: {}\n\n",
                    ParseError::InvalidArgumentAccountID {
                        name: "account_id".to_owned(),
                        e: AccountID::parse_str("test").unwrap_err()
                    }
                ),
                format!(
                    "Command: withdraw {} test\nStatus: error\nType: parse\nError: {}\n\n",
                    account_id.to_string(),
                    ParseError::InvalidArgumentUint {
                        name: "amount".to_owned(),
                        e: "test".parse::<u64>().unwrap_err(),
                    }
                ),
                format!("Bank: 1\nOpID: {}\nStatus: ok\n\n", operations[1].id),
                "Bank: 1\nStatus: error\nType: bank\nError: Insufficient funds\n\n".to_owned(),
            ]
            .join("")
        };

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

        let original_lock_context = Arc::new(RwLock::new(Context::default()));
        let lock_context = Arc::clone(&original_lock_context);

        handle(lock_context, &mut reader, &mut writer, &mut terminal).unwrap();

        let (account1_id, account2_id) = {
            let lock_context = Arc::clone(&original_lock_context);
            let context = lock_context.read().unwrap();

            let operations: Vec<&Operation> = context.banks[context.current_bank]
                .get_all_operations()
                .collect();

            (
                if let OperationKind::Register { id, .. } = operations[0].kind {
                    id
                } else {
                    AccountID::new()
                },
                if let OperationKind::Register { id, .. } = operations[1].kind {
                    id
                } else {
                    AccountID::new()
                },
            )
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

        let lock_context = Arc::clone(&original_lock_context);
        handle(lock_context, &mut reader, &mut writer, &mut terminal).unwrap();

        let expected = {
            let lock_context = Arc::clone(&original_lock_context);
            let context = lock_context.read().unwrap();
            let operations: Vec<&Operation> = context.banks[context.current_bank]
                .get_all_operations()
                .collect();

            vec![
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
                    ParseError::RequireArguments{
                        args: vec![
                            "sender_account_id".to_owned(),
                            "reciever_account_id".to_owned(),
                            "amount".to_owned()
                        ]
                    },
                ),
                format!(
                    "Command: transfer test1 test2 50\nStatus: error\nType: parse\nError: {}\n\n",
                    ParseError::InvalidArgumentAccountID{
                        name: "sender_account_id".to_owned(),
                        e: AccountID::parse_str("test1").unwrap_err()
                    }
                ),
                format!(
                    "Command: transfer {} test2 50\nStatus: error\nType: parse\nError: {}\n\n",
                    account1_id.to_string(),
                    ParseError::InvalidArgumentAccountID{
                        name: "reciever_account_id".to_owned(),
                        e: AccountID::parse_str("test2").unwrap_err()
                    }
                ),
                format!(
                    "Command: transfer test1 {} 50\nStatus: error\nType: parse\nError: {}\n\n",
                    account2_id.to_string(),
                    ParseError::InvalidArgumentAccountID{
                        name: "sender_account_id".to_owned(),
                        e: AccountID::parse_str("test1").unwrap_err()
                    }
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
            .join("")
        };

        assert_eq!(from_utf8(writer.as_slice()).unwrap(), expected);
    }

    #[test]
    fn handle_list_operations_works() {
        let input = vec![
            "register_account 100".to_owned(),
            "register_account 50".to_owned(),
        ]
        .join("\n");

        let mut reader = input.as_bytes();

        let mut writer = Vec::new();
        let mut terminal = Vec::new();

        let original_lock_context = Arc::new(RwLock::new(Context::default()));
        let lock_context = Arc::clone(&original_lock_context);

        handle(lock_context, &mut reader, &mut writer, &mut terminal).unwrap();

        let (account1_id, account2_id) = {
            let lock_context = Arc::clone(&original_lock_context);
            let context = lock_context.read().unwrap();
            let operations: Vec<&Operation> = context.banks[context.current_bank]
                .get_all_operations()
                .collect();

            (
                if let OperationKind::Register { id, .. } = operations[0].kind {
                    id
                } else {
                    AccountID::new()
                },
                if let OperationKind::Register { id, .. } = operations[1].kind {
                    id
                } else {
                    AccountID::new()
                },
            )
        };

        let input = vec![
            format!("deposit {} 100", account1_id.to_string()),
            format!("deposit {} 250", account2_id.to_string()),
            format!(
                "transfer {} {} 50",
                account1_id.to_string(),
                account2_id.to_string()
            ),
            format!("withdraw {} 50", account2_id.to_string()),
            "list_account_operations".to_owned(),
            "list_account_operations test".to_owned(),
            format!("list_account_operations {}", account1_id.to_string()),
            "list_all_operations".to_owned(),
        ]
        .join("\n");

        let mut reader = input.as_bytes();

        let lock_context = Arc::clone(&original_lock_context);
        handle(lock_context, &mut reader, &mut writer, &mut terminal).unwrap();

        let expected = {
            let lock_context = Arc::clone(&original_lock_context);
            let context = lock_context.read().unwrap();
            let operations: Vec<&Operation> = context.banks[context.current_bank]
                .get_all_operations()
                .collect();

            let account1_operations =
                context.banks[context.current_bank].get_account_operations(account1_id);

            vec![
                format!(
                    "Bank: 1\nOpID: {}\nStatus: ok\nResult: {}\n\n",
                    operations[0].id, account1_id,
                ),
                format!(
                    "Bank: 1\nOpID: {}\nStatus: ok\nResult: {}\n\n",
                    operations[1].id, account2_id,
                ),
                format!("Bank: 1\nOpID: {}\nStatus: ok\n\n", operations[2].id),
                format!("Bank: 1\nOpID: {}\nStatus: ok\n\n", operations[3].id),
                format!("Bank: 1\nOpID: {}\nStatus: ok\n\n", operations[4].id),
                format!("Bank: 1\nOpID: {}\nStatus: ok\n\n", operations[5].id),
                format!(
                    "Command: list_account_operations\nStatus: error\nType: parse\nError: {}\n\n",
                    ParseError::RequireArguments {
                        args: vec!["account_id".to_owned()]
                    },
                ),
                format!(
                    "Command: list_account_operations test\nStatus: error\nType: parse\nError: {}\n\n",
                    ParseError::InvalidArgumentAccountID {
                        name: "account_id".to_owned(),
                        e: AccountID::parse_str("test").unwrap_err()
                    },
                ),
                format!(
                    "Bank: 1\nStatus: ok\nResult: \n{}\n\n",
                    operations_as_string(account1_operations)
                ),
                format!(
                    "Bank: 1\nStatus: ok\nResult: \n{}\n\n",
                    operations_as_string(operations.into_iter())
                ),
            ]
            .join("")
        };

        assert_eq!(from_utf8(writer.as_slice()).unwrap(), expected);
    }

    #[test]
    fn handle_restore_bank_works() {
        let input = vec![
            "register_account 100".to_owned(),
            "register_account 50".to_owned(),
        ]
        .join("\n");

        let mut reader = input.as_bytes();

        let mut writer = Vec::new();
        let mut terminal = Vec::new();

        let original_lock_context = Arc::new(RwLock::new(Context::default()));
        let lock_context = Arc::clone(&original_lock_context);

        handle(lock_context, &mut reader, &mut writer, &mut terminal).unwrap();

        let (account1_id, account2_id) = {
            let lock_context = Arc::clone(&original_lock_context);
            let context = lock_context.read().unwrap();
            let operations: Vec<&Operation> = context.banks[context.current_bank]
                .get_all_operations()
                .collect();

            (
                if let OperationKind::Register { id, .. } = operations[0].kind {
                    id
                } else {
                    AccountID::new()
                },
                if let OperationKind::Register { id, .. } = operations[1].kind {
                    id
                } else {
                    AccountID::new()
                },
            )
        };

        let input = vec![
            format!("deposit {} 100", account1_id.to_string()),
            format!("deposit {} 250", account2_id.to_string()),
            format!(
                "transfer {} {} 50",
                account1_id.to_string(),
                account2_id.to_string()
            ),
            format!("withdraw {} 50", account2_id.to_string()),
            "restore_bank".to_owned(),
            "restore_bank test".to_owned(),
            "restore_bank 100".to_owned(),
            "restore_bank 1".to_owned(),
            "list_all_operations".to_owned(),
        ]
        .join("\n");

        let mut reader = input.as_bytes();

        let lock_context = Arc::clone(&original_lock_context);
        handle(lock_context, &mut reader, &mut writer, &mut terminal).unwrap();

        let expected = {
            let lock_context = Arc::clone(&original_lock_context);
            let context = lock_context.read().unwrap();

            let operations: Vec<&Operation> = context.banks[context.current_bank - 1]
                .get_all_operations()
                .collect();

            vec![
                format!(
                    "Bank: 1\nOpID: {}\nStatus: ok\nResult: {}\n\n",
                    operations[0].id, account1_id,
                ),
                format!(
                    "Bank: 1\nOpID: {}\nStatus: ok\nResult: {}\n\n",
                    operations[1].id, account2_id,
                ),
                format!("Bank: 1\nOpID: {}\nStatus: ok\n\n", operations[2].id),
                format!("Bank: 1\nOpID: {}\nStatus: ok\n\n", operations[3].id),
                format!("Bank: 1\nOpID: {}\nStatus: ok\n\n", operations[4].id),
                format!("Bank: 1\nOpID: {}\nStatus: ok\n\n", operations[5].id),
                format!(
                    "Command: restore_bank\nStatus: error\nType: parse\nError: {}\n\n",
                    ParseError::RequireArguments {
                        args: vec!["bank_id".to_owned()]
                    },
                ),
                format!(
                    "Command: restore_bank test\nStatus: error\nType: parse\nError: {}\n\n",
                    ParseError::InvalidArgumentUint {
                        name: "bank_id".to_owned(),
                        e: "test".parse::<u64>().unwrap_err(),
                    },
                ),
                "Bank: 1\nStatus: error\nType: bank\nError: invalid bank id\n\n".to_owned(),
                "Bank: 1\nStatus: ok\nResult: 2\n\n".to_owned(),
                format!(
                    "Bank: 2\nStatus: ok\nResult: \n{}\n\n",
                    operations_as_string(operations.into_iter())
                ),
            ]
            .join("")
        };

        assert_eq!(from_utf8(writer.as_slice()).unwrap(), expected);
    }
}
