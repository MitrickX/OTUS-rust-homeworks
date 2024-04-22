use crate::bank::account::AccountID;

#[derive(Debug, PartialEq)]
pub enum Command {
    NewBank,
    ChangeBank {
        id: u64,
    },
    RestoreBank {
        id: u64,
    },
    RegisterAccount {
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

#[derive(Debug, Clone, PartialEq)]
pub enum ParseError {
    EmptyCommand,
    RequireArguments(Vec<String>),
    InvalidArgumentUint(String, std::num::ParseIntError),
    InvalidArgumentAccountID(String, crate::bank::account::Error),
    UnknownCommand,
}

impl std::fmt::Display for ParseError {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        match self {
            ParseError::EmptyCommand => write!(f, "empty command"),
            ParseError::RequireArguments(args) => {
                write!(f, "require arguments: {}", args.join(", "))
            }
            ParseError::InvalidArgumentUint(name, e) => {
                write!(f, "invalid argument {name}: {e}")
            }
            ParseError::InvalidArgumentAccountID(name, e) => {
                write!(f, "invalid account {name}: {e}")
            }
            ParseError::UnknownCommand => {
                write!(f, "unknown command")
            }
        }
    }
}

impl std::error::Error for ParseError {}

pub type Result<T> = std::result::Result<T, ParseError>;

pub fn parse_argument_account_id(name: &str, value: &str) -> Result<AccountID> {
    AccountID::parse_str(value)
        .map_err(|e| ParseError::InvalidArgumentAccountID(name.to_string(), e))
}

pub fn parse_argument_uint(name: &str, value: &str) -> Result<u64> {
    value
        .parse()
        .map_err(|e| ParseError::InvalidArgumentUint(name.to_string(), e))
}

pub fn parse_command(command: &str) -> Result<Command> {
    let parts: Vec<&str> = command
        .split(' ')
        .map(|s| s.trim())
        .filter(|s| !s.is_empty())
        .collect();

    if parts.is_empty() {
        return Err(ParseError::EmptyCommand);
    }

    let command = parts[0];

    match command {
        "get_balance" | "list_account_operations" | "get_account_operations" => {
            if parts.len() < 2 {
                return Err(ParseError::RequireArguments(vec!["account_id".to_string()]));
            }

            match command {
                "get_balance" => Ok(Command::GetBalance {
                    id: parse_argument_account_id("account_id", parts[1])?,
                }),
                "list_account_operations" | "get_account_operations" => {
                    Ok(Command::ListAccountOperations {
                        id: parse_argument_account_id("account_id", parts[1])?,
                    })
                }
                _ => unreachable!(),
            }
        }
        "register_account" | "new_account" => {
            if parts.len() < 2 {
                return Err(ParseError::RequireArguments(vec!["balance".to_string()]));
            }

            Ok(Command::RegisterAccount {
                balance: parse_argument_uint("balance", parts[1])?,
            })
        }
        "deposit" | "withdraw" => {
            if parts.len() < 3 {
                return Err(ParseError::RequireArguments(vec![
                    "account_id".to_string(),
                    "ammount".to_string(),
                ]));
            }

            let id = parse_argument_account_id("account_id", parts[1])?;
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
                    "sender_account_id".to_string(),
                    "reciever_account_id".to_string(),
                    "ammount".to_string(),
                ]));
            }

            Ok(Command::Transfer {
                sender: parse_argument_account_id("sender_account_id", parts[1])?,
                reciever: parse_argument_account_id("reciever_account_id", parts[2])?,
                ammount: parse_argument_uint("ammount", parts[3])?,
            })
        }
        "change_bank" | "restore_bank" => {
            if parts.len() < 2 {
                return Err(ParseError::RequireArguments(vec!["bank_id".to_string()]));
            }

            let id = parse_argument_uint("bank_id", parts[1])?;

            match command {
                "change_bank" => Ok(Command::ChangeBank { id }),
                "restore_bank" => Ok(Command::RestoreBank { id }),
                _ => unreachable!(),
            }
        }
        "new_bank" => Ok(Command::NewBank),
        "list_all_operations" | "get_all_operations" => Ok(Command::ListAllOperations),
        "quit" => Ok(Command::Quit),
        _ => Err(ParseError::UnknownCommand),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parse_command_register_works() {
        assert_eq!(
            parse_command("register_account").unwrap_err(),
            ParseError::RequireArguments(vec!["balance".to_string()]),
        );

        assert_eq!(
            parse_command("register_account test").unwrap_err(),
            ParseError::InvalidArgumentUint(
                "balance".to_string(),
                "test".parse::<u64>().unwrap_err()
            ),
        );

        assert_eq!(
            parse_command("register_account 100").unwrap(),
            Command::RegisterAccount { balance: 100 },
        );
    }

    #[test]
    fn parse_command_get_balance_works() {
        assert_eq!(
            parse_command("get_balance").unwrap_err(),
            ParseError::RequireArguments(vec!["account_id".to_string()]),
        );

        assert_eq!(
            parse_command("get_balance test").unwrap_err(),
            ParseError::InvalidArgumentAccountID(
                "account_id".to_string(),
                AccountID::parse_str("test").unwrap_err()
            ),
        );

        assert_eq!(
            parse_command("get_balance 97c56a4e-0d75-4a82-b683-628b8c219fa3").unwrap(),
            Command::GetBalance {
                id: AccountID::parse_str("97c56a4e-0d75-4a82-b683-628b8c219fa3").unwrap()
            }
        );
    }

    #[test]
    fn parse_command_deposit_works() {
        assert_eq!(
            parse_command("deposit").unwrap_err(),
            ParseError::RequireArguments(vec!["account_id".to_string(), "ammount".to_string()]),
        );

        assert_eq!(
            parse_command("deposit test 123").unwrap_err(),
            ParseError::InvalidArgumentAccountID(
                "account_id".to_string(),
                AccountID::parse_str("test").unwrap_err()
            ),
        );

        assert_eq!(
            parse_command("deposit 97c56a4e-0d75-4a82-b683-628b8c219fa3 test").unwrap_err(),
            ParseError::InvalidArgumentUint(
                "ammount".to_string(),
                "test".parse::<u64>().unwrap_err(),
            )
        );

        assert_eq!(
            parse_command("deposit 97c56a4e-0d75-4a82-b683-628b8c219fa3 150").unwrap(),
            Command::Deposit {
                id: AccountID::parse_str("97c56a4e-0d75-4a82-b683-628b8c219fa3").unwrap(),
                balance: 150
            }
        );
    }

    #[test]
    fn parse_command_withdraw_works() {
        assert_eq!(
            parse_command("withdraw").unwrap_err(),
            ParseError::RequireArguments(vec!["account_id".to_string(), "ammount".to_string()]),
        );

        assert_eq!(
            parse_command("withdraw test 123").unwrap_err(),
            ParseError::InvalidArgumentAccountID(
                "account_id".to_string(),
                AccountID::parse_str("test").unwrap_err()
            ),
        );

        assert_eq!(
            parse_command("withdraw 97c56a4e-0d75-4a82-b683-628b8c219fa3 test").unwrap_err(),
            ParseError::InvalidArgumentUint(
                "ammount".to_string(),
                "test".parse::<u64>().unwrap_err(),
            )
        );

        assert_eq!(
            parse_command("withdraw 97c56a4e-0d75-4a82-b683-628b8c219fa3 150").unwrap(),
            Command::Withdraw {
                id: AccountID::parse_str("97c56a4e-0d75-4a82-b683-628b8c219fa3").unwrap(),
                balance: 150
            }
        );
    }

    #[test]
    fn parse_command_transfer_works() {
        assert_eq!(
            parse_command("transfer").unwrap_err(),
            ParseError::RequireArguments(vec![
                "sender_account_id".to_string(),
                "reciever_account_id".to_string(),
                "ammount".to_string()
            ]),
        );

        assert_eq!(
            parse_command("transfer from to 123").unwrap_err(),
            ParseError::InvalidArgumentAccountID(
                "sender_account_id".to_string(),
                AccountID::parse_str("from").unwrap_err()
            ),
        );

        assert_eq!(
            parse_command("transfer 97c56a4e-0d75-4a82-b683-628b8c219fa3 to 123").unwrap_err(),
            ParseError::InvalidArgumentAccountID(
                "reciever_account_id".to_string(),
                AccountID::parse_str("to").unwrap_err()
            )
        );

        assert_eq!(
            parse_command("transfer 97c56a4e-0d75-4a82-b683-628b8c219fa3 12c56a4e-0d75-5a82-b683-728d8c219fa3 test").unwrap_err(),
            ParseError::InvalidArgumentUint(
                "ammount".to_string(),
                "test".parse::<u64>().unwrap_err(),
            )
        );

        assert_eq!(
            parse_command("transfer 97c56a4e-0d75-4a82-b683-628b8c219fa3 12c56a4e-0d75-5a82-b683-728d8c219fa3 1000").unwrap(),
            Command::Transfer {
                sender: AccountID::parse_str("97c56a4e-0d75-4a82-b683-628b8c219fa3").unwrap(),
                reciever: AccountID::parse_str("12c56a4e-0d75-5a82-b683-728d8c219fa3").unwrap(),
                ammount: 1000
            }
        );
    }

    #[test]
    fn parse_command_list_account_operations_works() {
        assert_eq!(
            parse_command("list_account_operations").unwrap_err(),
            ParseError::RequireArguments(vec!["account_id".to_string()]),
        );

        assert_eq!(
            parse_command("list_account_operations test").unwrap_err(),
            ParseError::InvalidArgumentAccountID(
                "account_id".to_string(),
                AccountID::parse_str("test").unwrap_err()
            ),
        );

        assert_eq!(
            parse_command("list_account_operations 97c56a4e-0d75-4a82-b683-628b8c219fa3").unwrap(),
            Command::ListAccountOperations {
                id: AccountID::parse_str("97c56a4e-0d75-4a82-b683-628b8c219fa3").unwrap()
            }
        );
    }

    #[test]
    fn parse_command_list_all_operations_works() {
        assert_eq!(
            parse_command("list_all_operations").unwrap(),
            Command::ListAllOperations
        );
    }

    #[test]
    fn parse_command_change_bank_works() {
        assert_eq!(
            parse_command("change_bank").unwrap_err(),
            ParseError::RequireArguments(vec!["bank_id".to_string()]),
        );

        assert_eq!(
            parse_command("change_bank test").unwrap_err(),
            ParseError::InvalidArgumentUint(
                "bank_id".to_string(),
                "test".parse::<u64>().unwrap_err(),
            )
        );

        assert_eq!(
            parse_command("change_bank 123").unwrap(),
            Command::ChangeBank { id: 123 },
        );
    }

    #[test]
    fn parse_command_restore_bank_works() {
        assert_eq!(
            parse_command("restore_bank").unwrap_err(),
            ParseError::RequireArguments(vec!["bank_id".to_string()]),
        );

        assert_eq!(
            parse_command("restore_bank test").unwrap_err(),
            ParseError::InvalidArgumentUint(
                "bank_id".to_string(),
                "test".parse::<u64>().unwrap_err(),
            )
        );

        assert_eq!(
            parse_command("restore_bank 123").unwrap(),
            Command::RestoreBank { id: 123 },
        );
    }

    #[test]
    fn parse_command_new_bank_works() {
        assert_eq!(parse_command("new_bank").unwrap(), Command::NewBank);
    }

    #[test]
    fn parse_command_quit_works() {
        assert_eq!(parse_command("quit").unwrap(), Command::Quit);
    }

    #[test]
    fn parse_command_unknown_works() {
        assert_eq!(
            parse_command("some_abracadabra").unwrap_err(),
            ParseError::UnknownCommand,
        );
    }
}
