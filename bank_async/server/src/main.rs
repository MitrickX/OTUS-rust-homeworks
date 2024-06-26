use server::server::actor::repository_actor;
use server::server::command::Command;
use server::server::handler::handle;
use server::server::repository::Repository;
use tokio::{
    io::AsyncWriteExt,
    net::TcpListener,
    sync::{mpsc::unbounded_channel, oneshot::Sender},
};

type Result<T> = std::result::Result<T, Box<dyn std::error::Error + Send + Sync>>;

const ADDR: &str = "127.0.0.1:1337";

#[tokio::main]
async fn main() -> Result<()> {
    let listener = TcpListener::bind(ADDR).await?;

    println!("Listening on {}", listener.local_addr()?);

    let (sender, mut receiver) = unbounded_channel::<(Command, Sender<String>)>();

    tokio::spawn(async move {
        let mut repository = Repository::default();
        repository_actor(&mut repository, &mut receiver).await;
    });

    loop {
        let (mut stream, addr) = listener.accept().await?;

        println!("New client connected on {}", addr);

        let sender = sender.clone();
        tokio::spawn(async move {
            let (reader, mut writer) = stream.split();
            let mut terminal = std::io::stdout();

            writer
                .write_all(
                    br"Welcome to bank application
Print 'help' and press Enter to see the list of commands
",
                )
                .await
                .unwrap();

            match handle(&sender, reader, &mut writer, &mut terminal).await {
                Ok(_) => println!("{} disconnected", addr),
                Err(e) => {
                    writer
                        .write_all(
                            format!("Error occurred on server while handling request: {}\n", e)
                                .as_bytes(),
                        )
                        .await
                        .unwrap();

                    println!("Error occured: {}", e);
                }
            };
        });
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use regex::Regex;
    use server::bank::account::AccountID;
    use server::bank::log::OperationID;
    use server::server::command::ParseError;
    use std::str::from_utf8;

    #[tokio::test]
    async fn unknown_command_works() {
        let mut terminal = Vec::new();
        let (sender, _) = unbounded_channel::<(Command, Sender<String>)>();

        let reader = "test_command".as_bytes();
        let mut writer = Vec::new();

        handle(&sender, reader, &mut writer, &mut terminal)
            .await
            .unwrap();

        assert_eq!(
            "Command: test_command\nStatus: error\nType: parse\nError: unknown command\n\n",
            from_utf8(writer.as_slice()).unwrap()
        );
    }

    #[tokio::test]
    async fn handle_empty_command_works() {
        let mut terminal = Vec::new();

        let (sender, _) = unbounded_channel::<(Command, Sender<String>)>();

        let reader = "".as_bytes();
        let mut writer = Vec::new();

        handle(&sender, reader, &mut writer, &mut terminal)
            .await
            .unwrap();

        assert_eq!(
            from_utf8(terminal.as_slice()).unwrap(),
            "Client disconnected\n"
        );
    }

    #[tokio::test]
    async fn handle_quit_command_works() {
        let mut terminal = Vec::new();
        let (sender, _) = unbounded_channel::<(Command, Sender<String>)>();

        let reader = "quit".as_bytes();
        let mut writer = Vec::new();

        handle(&sender, reader, &mut writer, &mut terminal)
            .await
            .unwrap();

        assert_eq!(
            from_utf8(terminal.as_slice()).unwrap(),
            "Client quited\n".to_owned()
        );

        assert_eq!("Bye bye\n\n", from_utf8(writer.as_slice()).unwrap());
    }

    #[tokio::test]
    async fn handle_new_bank_command() {
        let mut terminal = Vec::new();
        let (sender, mut receiver) = unbounded_channel::<(Command, Sender<String>)>();

        let reader = "new_bank".as_bytes();
        let mut writer = Vec::new();

        tokio::spawn(async move {
            let mut repository = Repository::default();
            repository_actor(&mut repository, &mut receiver).await;
        });

        handle(&sender, reader, &mut writer, &mut terminal)
            .await
            .unwrap();

        assert_eq!(
            "Bank: 0\nStatus: ok\nResult: 1\n\n",
            from_utf8(writer.as_slice()).unwrap()
        );
    }

    #[tokio::test]
    async fn handle_which_bank_command() {
        let mut terminal = Vec::new();
        let (sender, mut receiver) = unbounded_channel::<(Command, Sender<String>)>();

        let input = vec!["which_bank", "new_bank", "which_bank"].join("\n");
        let reader = input.as_bytes();
        let mut writer = Vec::new();

        tokio::spawn(async move {
            let mut repository = Repository::default();
            repository_actor(&mut repository, &mut receiver).await;
        });

        handle(&sender, reader, &mut writer, &mut terminal)
            .await
            .unwrap();

        assert_eq!(
            vec![
                "Bank: 1\nStatus: ok\nResult: 1\n\n",
                "Bank: 1\nStatus: ok\nResult: 2\n\n",
                "Bank: 2\nStatus: ok\nResult: 2\n\n",
            ]
            .join(""),
            from_utf8(writer.as_slice()).unwrap()
        );
    }

    #[tokio::test]
    async fn handle_change_bank_command() {
        let mut terminal = Vec::new();
        let (sender, mut receiver) = unbounded_channel::<(Command, Sender<String>)>();

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
        let reader = input.as_bytes();
        let mut writer = Vec::new();

        tokio::spawn(async move {
            let mut repository = Repository::default();
            repository_actor(&mut repository, &mut receiver).await;
        });

        handle(&sender, reader, &mut writer, &mut terminal)
            .await
            .unwrap();

        assert_eq!(
            vec![
                "Bank: 0\nStatus: ok\nResult: 1\n\n",
                "Bank: 1\nStatus: ok\nResult: 2\n\n",
                "Bank: 2\nStatus: ok\nResult: 3\n\n",
                "Bank: 3\nStatus: ok\nResult: 2\n\n",
                "Bank: 2\nStatus: ok\nResult: 3\n\n",
                "Bank: 3\nStatus: ok\nResult: 1\n\n",
                "Bank: 1\nStatus: error\nType: bank\nError: invalid bank id\n\n",
                "Bank: 1\nStatus: error\nType: bank\nError: invalid bank id\n\n",
                "Bank: 1\nStatus: error\nType: bank\nError: invalid bank id\n\n",
            ]
            .join(""),
            from_utf8(writer.as_slice()).unwrap()
        );
    }

    #[tokio::test]
    async fn handle_register_account_works() {
        let mut terminal = Vec::new();
        let (sender, mut receiver) = unbounded_channel::<(Command, Sender<String>)>();

        let input = vec!["register_account", "register_account 100"].join("\n");
        let reader = input.as_bytes();
        let mut writer = Vec::new();

        tokio::spawn(async move {
            let mut repository = Repository::default();
            repository_actor(&mut repository, &mut receiver).await;
        });

        handle(&sender, reader, &mut writer, &mut terminal)
            .await
            .unwrap();

        let result = from_utf8(writer.as_slice())
            .unwrap()
            .to_owned()
            .split("\n\n")
            .map(|s| s.to_owned())
            .collect::<Vec<_>>();

        assert_eq!(
            format!(
                "Command: register_account\nStatus: error\nType: parse\nError: {}",
                ParseError::RequireArguments {
                    args: vec!["balance".to_string()]
                },
            ),
            result[0]
        );

        let re_pattern = r"Bank: 1\nOpID: ([a-f0-9-]+)\nStatus: ok\nResult: ([a-f0-9-]+)";
        let re = Regex::new(re_pattern).unwrap();

        assert!(re.is_match(&result[1]));

        let (_, [operation_id, account_id]) = re.captures(&result[1]).unwrap().extract();

        assert!(OperationID::parse_str(operation_id).is_ok());
        assert!(AccountID::parse_str(account_id).is_ok());
    }

    #[tokio::test]
    async fn handle_get_balance_works() {
        let reader = "register_account 100".as_bytes();
        let mut writer = Vec::new();

        let mut terminal = Vec::new();

        let (sender, mut receiver) = unbounded_channel::<(Command, Sender<String>)>();

        tokio::spawn(async move {
            let mut repository = Repository::default();
            repository_actor(&mut repository, &mut receiver).await;
        });

        handle(&sender, reader, &mut writer, &mut terminal)
            .await
            .unwrap();

        let result = from_utf8(writer.as_slice())
            .unwrap()
            .to_owned()
            .split("\n\n")
            .map(|s| s.to_owned())
            .collect::<Vec<_>>();

        let re_pattern = r"Bank: 1\nOpID: ([a-f0-9-]+)\nStatus: ok\nResult: ([a-f0-9-]+)";
        let re = Regex::new(re_pattern).unwrap();

        assert!(re.is_match(&result[0]));

        let (_, [operation_id, account_id]) = re.captures(&result[0]).unwrap().extract();

        assert!(OperationID::parse_str(operation_id).is_ok());
        assert!(AccountID::parse_str(account_id).is_ok());

        let input = vec![
            "get_balance".to_owned(),
            "get_balance test".to_owned(),
            format!("get_balance {}", account_id.to_string()),
        ]
        .join("\n");

        let mut reader = input.as_bytes();

        handle(&sender, &mut reader, &mut writer, &mut terminal)
            .await
            .unwrap();

        let result = from_utf8(writer.as_slice())
            .unwrap()
            .to_owned()
            .split("\n\n")
            .map(|s| s.to_owned())
            .collect::<Vec<_>>();

        assert_eq!(
            format!(
                "Command: get_balance\nStatus: error\nType: parse\nError: {}",
                ParseError::RequireArguments {
                    args: vec!["account_id".to_string()]
                },
            ),
            result[1]
        );

        assert_eq!(
            format!(
                "Command: get_balance test\nStatus: error\nType: parse\nError: {}",
                ParseError::InvalidArgumentAccountID {
                    name: "account_id".to_owned(),
                    e: AccountID::parse_str("test").unwrap_err()
                }
            ),
            result[2]
        );

        assert_eq!(format!("Bank: 1\nStatus: ok\nResult: {}", 100), result[3]);
    }

    #[tokio::test]
    async fn handle_get_balance_when_no_accounts_yet_works() {
        let reader = "get_balance 6fce6320-256a-4459-a1d3-5da077e21661".as_bytes();
        let mut writer = Vec::new();

        let mut terminal = Vec::new();

        let (sender, mut receiver) = unbounded_channel::<(Command, Sender<String>)>();

        tokio::spawn(async move {
            let mut repository = Repository::default();
            repository_actor(&mut repository, &mut receiver).await;
        });

        handle(&sender, reader, &mut writer, &mut terminal)
            .await
            .unwrap();

        assert_eq!(
            "Bank: 1\nStatus: fail\nResult: Bank error: Account not found\n\n",
            from_utf8(writer.as_slice()).unwrap()
        );
    }

    #[tokio::test]
    async fn handle_deposit_works() {
        let reader = "register_account 100".as_bytes();
        let mut writer = Vec::new();

        let mut terminal = Vec::new();

        let (sender, mut receiver) = unbounded_channel::<(Command, Sender<String>)>();

        tokio::spawn(async move {
            let mut repository = Repository::default();
            repository_actor(&mut repository, &mut receiver).await;
        });

        handle(&sender, reader, &mut writer, &mut terminal)
            .await
            .unwrap();

        let result = from_utf8(writer.as_slice())
            .unwrap()
            .to_owned()
            .split("\n\n")
            .map(|s| s.to_owned())
            .collect::<Vec<_>>();

        let re_pattern = r"Bank: 1\nOpID: ([a-f0-9-]+)\nStatus: ok\nResult: ([a-f0-9-]+)";
        let re = Regex::new(re_pattern).unwrap();

        assert!(re.is_match(&result[0]));

        let (_, [operation_id, account_id]) = re.captures(&result[0]).unwrap().extract();

        assert!(OperationID::parse_str(operation_id).is_ok());
        assert!(AccountID::parse_str(account_id).is_ok());

        let input = vec![
            "deposit".to_owned(),
            "deposit test 10".to_owned(),
            format!("deposit {} test", account_id.to_string()),
            format!("deposit {} 100", account_id.to_string()),
        ]
        .join("\n");

        let reader = input.as_bytes();

        handle(&sender, reader, &mut writer, &mut terminal)
            .await
            .unwrap();

        let result = from_utf8(writer.as_slice())
            .unwrap()
            .to_owned()
            .split("\n\n")
            .map(|s| s.to_owned())
            .collect::<Vec<_>>();

        assert_eq!(
            format!(
                "Command: deposit\nStatus: error\nType: parse\nError: {}",
                ParseError::RequireArguments {
                    args: vec!["account_id".to_owned(), "amount".to_owned()]
                },
            ),
            result[1]
        );

        assert_eq!(
            format!(
                "Command: deposit test 10\nStatus: error\nType: parse\nError: {}",
                ParseError::InvalidArgumentAccountID {
                    name: "account_id".to_owned(),
                    e: AccountID::parse_str("test").unwrap_err()
                }
            ),
            result[2]
        );

        assert_eq!(
            format!(
                "Command: deposit {} test\nStatus: error\nType: parse\nError: {}",
                account_id.to_string(),
                ParseError::InvalidArgumentUint {
                    name: "amount".to_owned(),
                    e: "test".parse::<u64>().unwrap_err(),
                }
            ),
            result[3]
        );

        let re_pattern = r"Bank: 1\nOpID: ([a-f0-9-]+)\nStatus: ok";
        let re = Regex::new(re_pattern).unwrap();

        assert!(re.is_match(&result[4]));

        let (_, [operation_id]) = re.captures(&result[0]).unwrap().extract();

        assert!(OperationID::parse_str(operation_id).is_ok());
    }

    #[tokio::test]
    async fn handle_withdraw_works() {
        let reader = "register_account 100".as_bytes();

        let mut writer = Vec::new();
        let mut terminal = Vec::new();

        let (sender, mut receiver) = unbounded_channel::<(Command, Sender<String>)>();

        tokio::spawn(async move {
            let mut repository = Repository::default();
            repository_actor(&mut repository, &mut receiver).await;
        });

        handle(&sender, reader, &mut writer, &mut terminal)
            .await
            .unwrap();

        let result = from_utf8(writer.as_slice())
            .unwrap()
            .to_owned()
            .split("\n\n")
            .map(|s| s.to_owned())
            .collect::<Vec<_>>();

        let re_pattern = r"Bank: 1\nOpID: ([a-f0-9-]+)\nStatus: ok\nResult: ([a-f0-9-]+)";
        let re = Regex::new(re_pattern).unwrap();

        assert!(re.is_match(&result[0]));

        let (_, [operation_id, account_id]) = re.captures(&result[0]).unwrap().extract();

        assert!(OperationID::parse_str(operation_id).is_ok());
        assert!(AccountID::parse_str(account_id).is_ok());

        let input = vec![
            "withdraw".to_owned(),
            "withdraw test 10".to_owned(),
            format!("withdraw {} test", account_id.to_string()),
            format!("withdraw {} 100", account_id.to_string()),
            format!("withdraw {} 100", account_id.to_string()),
        ]
        .join("\n");

        let reader = input.as_bytes();

        handle(&sender, reader, &mut writer, &mut terminal)
            .await
            .unwrap();

        let result = from_utf8(writer.as_slice())
            .unwrap()
            .to_owned()
            .split("\n\n")
            .map(|s| s.to_owned())
            .collect::<Vec<_>>();

        assert_eq!(
            format!(
                "Command: withdraw\nStatus: error\nType: parse\nError: {}",
                ParseError::RequireArguments {
                    args: vec!["account_id".to_owned(), "amount".to_owned()]
                },
            ),
            result[1]
        );

        assert_eq!(
            format!(
                "Command: withdraw test 10\nStatus: error\nType: parse\nError: {}",
                ParseError::InvalidArgumentAccountID {
                    name: "account_id".to_owned(),
                    e: AccountID::parse_str("test").unwrap_err()
                }
            ),
            result[2]
        );

        assert_eq!(
            format!(
                "Command: withdraw {} test\nStatus: error\nType: parse\nError: {}",
                account_id.to_string(),
                ParseError::InvalidArgumentUint {
                    name: "amount".to_owned(),
                    e: "test".parse::<u64>().unwrap_err(),
                }
            ),
            result[3]
        );

        let re_pattern = r"Bank: 1\nOpID: ([a-f0-9-]+)\nStatus: ok";
        let re = Regex::new(re_pattern).unwrap();

        assert!(re.is_match(&result[4]));

        let (_, [operation_id]) = re.captures(&result[4]).unwrap().extract();

        assert!(OperationID::parse_str(operation_id).is_ok());

        assert_eq!(
            "Bank: 1\nStatus: error\nType: bank\nError: Bank error: Insufficient funds".to_owned(),
            result[5]
        );
    }

    #[tokio::test]
    async fn handle_transfer_works() {
        let input = vec![
            "register_account 100".to_owned(),
            "register_account 50".to_owned(),
        ]
        .join("\n");

        let reader = input.as_bytes();

        let mut writer = Vec::new();
        let mut terminal = Vec::new();

        let (sender, mut receiver) = unbounded_channel::<(Command, Sender<String>)>();

        tokio::spawn(async move {
            let mut repository = Repository::default();
            repository_actor(&mut repository, &mut receiver).await;
        });

        handle(&sender, reader, &mut writer, &mut terminal)
            .await
            .unwrap();

        let result = from_utf8(writer.as_slice())
            .unwrap()
            .to_owned()
            .split("\n\n")
            .map(|s| s.to_owned())
            .collect::<Vec<_>>();

        let re_pattern = r"Bank: 1\nOpID: ([a-f0-9-]+)\nStatus: ok\nResult: ([a-f0-9-]+)";
        let re = Regex::new(re_pattern).unwrap();

        assert!(re.is_match(&result[0]));
        assert!(re.is_match(&result[1]));

        let (_, [operation1_id, account1_id]) = re.captures(&result[0]).unwrap().extract();
        let (_, [operation2_id, account2_id]) = re.captures(&result[1]).unwrap().extract();

        assert!(OperationID::parse_str(operation1_id).is_ok());
        assert!(AccountID::parse_str(account1_id).is_ok());

        assert!(OperationID::parse_str(operation2_id).is_ok());
        assert!(AccountID::parse_str(account2_id).is_ok());

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

        let reader = input.as_bytes();

        handle(&sender, reader, &mut writer, &mut terminal)
            .await
            .unwrap();

        let result = from_utf8(writer.as_slice())
            .unwrap()
            .to_owned()
            .split("\n\n")
            .map(|s| s.to_owned())
            .collect::<Vec<_>>();

        assert_eq!(
            format!(
                "Command: transfer\nStatus: error\nType: parse\nError: {}",
                ParseError::RequireArguments {
                    args: vec![
                        "sender_account_id".to_owned(),
                        "receiver_account_id".to_owned(),
                        "amount".to_owned()
                    ]
                },
            ),
            result[2]
        );

        assert_eq!(
            format!(
                "Command: transfer test1 test2 50\nStatus: error\nType: parse\nError: {}",
                ParseError::InvalidArgumentAccountID {
                    name: "sender_account_id".to_owned(),
                    e: AccountID::parse_str("test1").unwrap_err()
                }
            ),
            result[3]
        );

        assert_eq!(
            format!(
                "Command: transfer {} test2 50\nStatus: error\nType: parse\nError: {}",
                account1_id.to_string(),
                ParseError::InvalidArgumentAccountID {
                    name: "receiver_account_id".to_owned(),
                    e: AccountID::parse_str("test2").unwrap_err()
                }
            ),
            result[4]
        );

        assert_eq!(
            format!(
                "Command: transfer test1 {} 50\nStatus: error\nType: parse\nError: {}",
                account2_id.to_string(),
                ParseError::InvalidArgumentAccountID {
                    name: "sender_account_id".to_owned(),
                    e: AccountID::parse_str("test1").unwrap_err()
                }
            ),
            result[5]
        );

        assert_eq!(
            format!(
                "Command: transfer {} {} test\nStatus: error\nType: parse\nError: invalid argument amount: {}",
                account1_id.to_string(),
                account2_id.to_string(),
                "test".parse::<u64>().unwrap_err(),
            ),
            result[6]
        );

        let re_pattern = r"Bank: 1\nOpID: ([a-f0-9-]+)\nStatus: ok";
        let re = Regex::new(re_pattern).unwrap();

        assert!(re.is_match(&result[7]));

        let (_, [operation_id]) = re.captures(&result[0]).unwrap().extract();

        assert!(OperationID::parse_str(operation_id).is_ok());

        assert_eq!(
            "Bank: 1\nStatus: error\nType: bank\nError: Bank error: Insufficient funds".to_owned(),
            result[8]
        );
    }

    #[tokio::test]
    async fn handle_list_operations_empty_case_works() {
        let input = vec!["get_all_operations"].join("\n");
        let reader = input.as_bytes();
        let mut writer = Vec::new();
        let mut terminal = Vec::new();

        let (sender, mut receiver) = unbounded_channel::<(Command, Sender<String>)>();

        tokio::spawn(async move {
            let mut repository = Repository::default();
            repository_actor(&mut repository, &mut receiver).await;
        });

        handle(&sender, reader, &mut writer, &mut terminal)
            .await
            .unwrap();

        let result = from_utf8(writer.as_slice()).unwrap();

        assert_eq!(
            "Bank: 0\nStatus: ok\nResult:\nno operations yet\n\n",
            result
        );
    }

    #[tokio::test]
    async fn handle_list_operations_works() {
        let input = vec!["register_account 100", "register_account 50"].join("\n");

        let reader = input.as_bytes();

        let mut writer = Vec::new();
        let mut terminal = Vec::new();

        let (sender, mut receiver) = unbounded_channel::<(Command, Sender<String>)>();

        tokio::spawn(async move {
            let mut repository = Repository::default();
            repository_actor(&mut repository, &mut receiver).await;
        });

        handle(&sender, reader, &mut writer, &mut terminal)
            .await
            .unwrap();

        let result = from_utf8(writer.as_slice())
            .unwrap()
            .to_owned()
            .split("\n\n")
            .map(|s| s.to_owned())
            .collect::<Vec<_>>();

        let re_pattern = r"Bank: 1\nOpID: ([a-f0-9-]+)\nStatus: ok\nResult: ([a-f0-9-]+)";
        let re = Regex::new(re_pattern).unwrap();

        assert!(re.is_match(&result[0]));
        assert!(re.is_match(&result[1]));

        let (_, [operation1_id, account1_id]) = re.captures(&result[0]).unwrap().extract();
        let (_, [operation2_id, account2_id]) = re.captures(&result[1]).unwrap().extract();

        assert!(OperationID::parse_str(operation1_id).is_ok());
        assert!(AccountID::parse_str(account1_id).is_ok());

        assert!(OperationID::parse_str(operation2_id).is_ok());
        assert!(AccountID::parse_str(account2_id).is_ok());

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

        let reader = input.as_bytes();

        handle(&sender, reader, &mut writer, &mut terminal)
            .await
            .unwrap();

        let result = from_utf8(writer.as_slice())
            .unwrap()
            .to_owned()
            .split("\n\n")
            .map(|s| s.to_owned())
            .collect::<Vec<_>>();

        let re_pattern = r"Bank: 1\nOpID: ([a-f0-9-]+)\nStatus: ok";
        let re = Regex::new(re_pattern).unwrap();

        assert!(re.is_match(&result[2]));
        assert!(re.is_match(&result[3]));
        assert!(re.is_match(&result[4]));
        assert!(re.is_match(&result[5]));

        assert_eq!(
            format!(
                "Command: list_account_operations\nStatus: error\nType: parse\nError: {}",
                ParseError::RequireArguments {
                    args: vec!["account_id".to_owned()]
                },
            ),
            result[6]
        );

        assert_eq!(
            format!(
                "Command: list_account_operations test\nStatus: error\nType: parse\nError: {}",
                ParseError::InvalidArgumentAccountID {
                    name: "account_id".to_owned(),
                    e: AccountID::parse_str("test").unwrap_err()
                },
            ),
            result[7]
        );

        let re_pattern = r"Bank: 1\nStatus: ok\nResult:\n(.*)\n(.*)\n(.*)";
        let re = Regex::new(re_pattern).unwrap();

        assert!(re.is_match(&result[8]));

        let (_, [operation1, operation2, operation3]) = re.captures(&result[8]).unwrap().extract();

        let re = Regex::new(r"([a-f0-9-]+): \(Register ([a-f0-9-]+) 100\)").unwrap();
        assert!(re.is_match(&operation1));

        let (_, [operation_id, account_id]) = re.captures(&operation1).unwrap().extract();
        assert!(OperationID::parse_str(operation_id).is_ok());
        assert!(AccountID::parse_str(account_id).is_ok());

        assert_eq!(account1_id, account_id);

        let re = Regex::new(r"([a-f0-9-]+): \(Deposit ([a-f0-9-]+) 100\)").unwrap();
        assert!(re.is_match(&operation2));

        let (_, [operation_id, account_id]) = re.captures(&operation2).unwrap().extract();
        assert!(OperationID::parse_str(operation_id).is_ok());
        assert!(AccountID::parse_str(account_id).is_ok());

        assert_eq!(account1_id, account_id);

        let re = Regex::new(r"([a-f0-9-]+): \(Transfer ([a-f0-9-]+) ([a-f0-9-]+) 50\)").unwrap();
        assert!(re.is_match(&operation3));

        let (_, [operation_id, sender_account_id, receiver_account_id]) =
            re.captures(&operation3).unwrap().extract();
        assert!(OperationID::parse_str(operation_id).is_ok());
        assert!(AccountID::parse_str(sender_account_id).is_ok());
        assert!(AccountID::parse_str(receiver_account_id).is_ok());

        assert_eq!(account1_id, sender_account_id);
        assert_eq!(account2_id, receiver_account_id);

        let re_pattern = r"Bank: 1\nStatus: ok\nResult:\n(.*)\n(.*)\n(.*)\n(.*)\n(.*)\n(.*)";
        let re = Regex::new(re_pattern).unwrap();

        assert!(re.is_match(&result[9]));

        let (_, [operation1, operation2, operation3, operation4, operation5, operation6]) =
            re.captures(&result[9]).unwrap().extract();

        let re = Regex::new(r"([a-f0-9-]+): \(Register ([a-f0-9-]+) 100\)").unwrap();
        assert!(re.is_match(&operation1));

        let (_, [operation_id, account_id]) = re.captures(&operation1).unwrap().extract();
        assert!(OperationID::parse_str(operation_id).is_ok());
        assert!(AccountID::parse_str(account_id).is_ok());

        assert_eq!(account1_id, account_id);

        let re = Regex::new(r"([a-f0-9-]+): \(Register ([a-f0-9-]+) 50\)").unwrap();
        assert!(re.is_match(&operation2));

        let (_, [operation_id, account_id]) = re.captures(&operation2).unwrap().extract();
        assert!(OperationID::parse_str(operation_id).is_ok());
        assert!(AccountID::parse_str(account_id).is_ok());

        assert_eq!(account2_id, account_id);

        let re = Regex::new(r"([a-f0-9-]+): \(Deposit ([a-f0-9-]+) 100\)").unwrap();
        assert!(re.is_match(&operation3));

        let (_, [operation_id, account_id]) = re.captures(&operation3).unwrap().extract();
        assert!(OperationID::parse_str(operation_id).is_ok());
        assert!(AccountID::parse_str(account_id).is_ok());

        assert_eq!(account1_id, account_id);

        let re = Regex::new(r"([a-f0-9-]+): \(Deposit ([a-f0-9-]+) 250\)").unwrap();
        assert!(re.is_match(&operation4));

        let (_, [operation_id, account_id]) = re.captures(&operation4).unwrap().extract();
        assert!(OperationID::parse_str(operation_id).is_ok());
        assert!(AccountID::parse_str(account_id).is_ok());

        assert_eq!(account2_id, account_id);

        let re = Regex::new(r"([a-f0-9-]+): \(Transfer ([a-f0-9-]+) ([a-f0-9-]+) 50\)").unwrap();
        assert!(re.is_match(&operation5));

        let (_, [operation_id, sender_account_id, receiver_account_id]) =
            re.captures(&operation5).unwrap().extract();
        assert!(OperationID::parse_str(operation_id).is_ok());
        assert!(AccountID::parse_str(sender_account_id).is_ok());
        assert!(AccountID::parse_str(receiver_account_id).is_ok());

        assert_eq!(account1_id, sender_account_id);
        assert_eq!(account2_id, receiver_account_id);

        let re = Regex::new(r"([a-f0-9-]+): \(Withdraw ([a-f0-9-]+) 50\)").unwrap();
        assert!(re.is_match(&operation6));

        let (_, [operation_id, account_id]) = re.captures(&operation6).unwrap().extract();
        assert!(OperationID::parse_str(operation_id).is_ok());
        assert!(AccountID::parse_str(account_id).is_ok());

        assert_eq!(account2_id, account_id);
    }

    #[tokio::test]
    async fn handle_restore_bank_works() {
        let input = vec![
            "register_account 100".to_owned(),
            "register_account 50".to_owned(),
        ]
        .join("\n");

        let reader = input.as_bytes();

        let mut writer = Vec::new();
        let mut terminal = Vec::new();

        let (sender, mut receiver) = unbounded_channel::<(Command, Sender<String>)>();

        tokio::spawn(async move {
            let mut repository = Repository::default();
            repository_actor(&mut repository, &mut receiver).await;
        });

        handle(&sender, reader, &mut writer, &mut terminal)
            .await
            .unwrap();

        let result = from_utf8(writer.as_slice())
            .unwrap()
            .to_owned()
            .split("\n\n")
            .map(|s| s.to_owned())
            .collect::<Vec<_>>();

        let re_pattern = r"Bank: 1\nOpID: ([a-f0-9-]+)\nStatus: ok\nResult: ([a-f0-9-]+)";
        let re = Regex::new(re_pattern).unwrap();

        assert!(re.is_match(&result[0]));
        assert!(re.is_match(&result[1]));

        let (_, [operation1_id, account1_id]) = re.captures(&result[0]).unwrap().extract();
        let (_, [operation2_id, account2_id]) = re.captures(&result[1]).unwrap().extract();

        assert!(OperationID::parse_str(operation1_id).is_ok());
        assert!(AccountID::parse_str(account1_id).is_ok());

        assert!(OperationID::parse_str(operation2_id).is_ok());
        assert!(AccountID::parse_str(account2_id).is_ok());

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

        handle(&sender, &mut reader, &mut writer, &mut terminal)
            .await
            .unwrap();

        let result = from_utf8(writer.as_slice())
            .unwrap()
            .to_owned()
            .split("\n\n")
            .map(|s| s.to_owned())
            .collect::<Vec<_>>();

        let re_pattern = r"Bank: 1\nOpID: ([a-f0-9-]+)\nStatus: ok";
        let re = Regex::new(re_pattern).unwrap();

        assert!(re.is_match(&result[2]));
        assert!(re.is_match(&result[3]));
        assert!(re.is_match(&result[4]));
        assert!(re.is_match(&result[5]));

        assert_eq!(
            format!(
                "Command: restore_bank\nStatus: error\nType: parse\nError: {}",
                ParseError::RequireArguments {
                    args: vec!["bank_id".to_owned()]
                },
            ),
            result[6]
        );

        assert_eq!(
            format!(
                "Command: restore_bank test\nStatus: error\nType: parse\nError: {}",
                ParseError::InvalidArgumentUint {
                    name: "bank_id".to_owned(),
                    e: "test".parse::<u64>().unwrap_err(),
                },
            ),
            result[7]
        );

        assert_eq!(
            "Bank: 1\nStatus: error\nType: bank\nError: invalid bank id".to_owned(),
            result[8]
        );

        assert_eq!("Bank: 1\nStatus: ok\nResult: 2".to_owned(), result[9]);

        println!("{}", result[10]);

        let re_pattern = r"Bank: 2\nStatus: ok\nResult:\n(.*)\n(.*)\n(.*)\n(.*)\n(.*)\n(.*)";
        let re = Regex::new(re_pattern).unwrap();

        assert!(re.is_match(&result[10]));

        let (_, [operation1, operation2, operation3, operation4, operation5, operation6]) =
            re.captures(&result[10]).unwrap().extract();

        let re = Regex::new(r"([a-f0-9-]+): \(Register ([a-f0-9-]+) 100\)").unwrap();
        assert!(re.is_match(&operation1));

        let (_, [operation_id, account_id]) = re.captures(&operation1).unwrap().extract();
        assert!(OperationID::parse_str(operation_id).is_ok());
        assert!(AccountID::parse_str(account_id).is_ok());

        assert_eq!(account1_id, account_id);

        let re = Regex::new(r"([a-f0-9-]+): \(Register ([a-f0-9-]+) 50\)").unwrap();
        assert!(re.is_match(&operation2));

        let (_, [operation_id, account_id]) = re.captures(&operation2).unwrap().extract();
        assert!(OperationID::parse_str(operation_id).is_ok());
        assert!(AccountID::parse_str(account_id).is_ok());

        assert_eq!(account2_id, account_id);

        let re = Regex::new(r"([a-f0-9-]+): \(Deposit ([a-f0-9-]+) 100\)").unwrap();
        assert!(re.is_match(&operation3));

        let (_, [operation_id, account_id]) = re.captures(&operation3).unwrap().extract();
        assert!(OperationID::parse_str(operation_id).is_ok());
        assert!(AccountID::parse_str(account_id).is_ok());

        assert_eq!(account1_id, account_id);

        let re = Regex::new(r"([a-f0-9-]+): \(Deposit ([a-f0-9-]+) 250\)").unwrap();
        assert!(re.is_match(&operation4));

        let (_, [operation_id, account_id]) = re.captures(&operation4).unwrap().extract();
        assert!(OperationID::parse_str(operation_id).is_ok());
        assert!(AccountID::parse_str(account_id).is_ok());

        assert_eq!(account2_id, account_id);

        let re = Regex::new(r"([a-f0-9-]+): \(Transfer ([a-f0-9-]+) ([a-f0-9-]+) 50\)").unwrap();
        assert!(re.is_match(&operation5));

        let (_, [operation_id, sender_account_id, receiver_account_id]) =
            re.captures(&operation5).unwrap().extract();
        assert!(OperationID::parse_str(operation_id).is_ok());
        assert!(AccountID::parse_str(sender_account_id).is_ok());
        assert!(AccountID::parse_str(receiver_account_id).is_ok());

        assert_eq!(account1_id, sender_account_id);
        assert_eq!(account2_id, receiver_account_id);

        let re = Regex::new(r"([a-f0-9-]+): \(Withdraw ([a-f0-9-]+) 50\)").unwrap();
        assert!(re.is_match(&operation6));

        let (_, [operation_id, account_id]) = re.captures(&operation6).unwrap().extract();
        assert!(OperationID::parse_str(operation_id).is_ok());
        assert!(AccountID::parse_str(account_id).is_ok());

        assert_eq!(account2_id, account_id);
    }
}
