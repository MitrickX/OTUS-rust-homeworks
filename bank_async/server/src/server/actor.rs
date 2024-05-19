use crate::bank::account::AccountID;
use crate::bank::log::Operation;
use crate::server::command::Command;
use crate::server::repository::{Repository, RepositoryError};
use tokio::sync::{mpsc::UnboundedReceiver, oneshot::Sender};

pub async fn repository_actor(
    repository: &mut Repository,
    command_receiver: &mut UnboundedReceiver<(Command, Sender<String>)>,
) {
    loop {
        if let Some((command, response_sender)) = command_receiver.recv().await {
            let response = handle_command(repository, &command);
            if let Err(err) = response_sender.send(response) {
                eprintln!("Error sending response: {}", err);
            }
        }
    }
}

fn handle_new_bank(repository: &mut Repository) -> String {
    let bank_id = repository.new_bank();
    format!("Bank: {}\nStatus: ok\nResult: {}\n\n", bank_id - 1, bank_id)
}

fn handle_change_bank(repository: &mut Repository, id: u64) -> String {
    let current_bank_id = repository.current_bank_id();
    match repository.change_bank(id) {
        Ok(_) => format!("Bank: {}\nStatus: ok\nResult: {}\n\n", current_bank_id, id),
        Err(_) => format!(
            "Bank: {}\nStatus: error\nType: bank\nError: invalid bank id\n\n",
            current_bank_id,
        ),
    }
}

fn handle_which_bank(repository: &mut Repository) -> String {
    if repository.current_bank_id() == 0 {
        repository.new_bank();
    }

    let current_bank = repository.current_bank_id();
    format!(
        "Bank: {}\nStatus: ok\nResult: {}\n\n",
        current_bank, current_bank
    )
}

fn handle_restore_bank(repository: &mut Repository, id: u64) -> String {
    let current_bank = repository.current_bank_id();

    match repository.restore_bank(id) {
        Ok(_) => format!(
            "Bank: {}\nStatus: ok\nResult: {}\n\n",
            current_bank,
            repository.current_bank_id(),
        ),
        Err(repository_err) => match repository_err {
            RepositoryError::InvalidBankId => format!(
                "Bank: {}\nStatus: error\nType: bank\nError: invalid bank id\n\n",
                current_bank,
            ),
            RepositoryError::BankError(e) => format!(
                "Bank: {}\nStatus: error\nType: bank\nError: {}\n\n",
                current_bank, e,
            ),
        },
    }
}

fn handle_register_account(repository: &mut Repository, balance: u64) -> String {
    match repository.register_account(balance) {
        Ok((account_id, opperation_id)) => {
            format!(
                "Bank: {}\nOpID: {}\nStatus: ok\nResult: {}\n\n",
                repository.current_bank_id(),
                opperation_id,
                account_id
            )
        }
        Err(e) => {
            format!(
                "Bank: {}\nStatus: error\nType: bank\nError: {}\n\n",
                repository.current_bank_id(),
                e
            )
        }
    }
}

fn handle_get_balance(repository: &mut Repository, id: AccountID) -> String {
    match repository.get_balance(id) {
        Ok(balance) => {
            format!(
                "Bank: {}\nStatus: ok\nResult: {}\n\n",
                repository.current_bank_id(),
                balance
            )
        }
        Err(e) => {
            format!(
                "Bank: {}\nStatus: fail\nResult: {}\n\n",
                repository.current_bank_id(),
                e
            )
        }
    }
}

fn handle_deposit(repository: &mut Repository, id: AccountID, amount: u64) -> String {
    match repository.deposit(id, amount) {
        Ok(opperation_id) => {
            format!(
                "Bank: {}\nOpID: {}\nStatus: ok\n\n",
                repository.current_bank_id(),
                opperation_id,
            )
        }
        Err(e) => {
            format!(
                "Bank: {}\nStatus: fail\nResult: {}\n\n",
                repository.current_bank_id(),
                e
            )
        }
    }
}

fn handle_withdraw(repository: &mut Repository, id: AccountID, amount: u64) -> String {
    match repository.withdraw(id, amount) {
        Ok(opperation_id) => {
            format!(
                "Bank: {}\nOpID: {}\nStatus: ok\n\n",
                repository.current_bank_id(),
                opperation_id,
            )
        }
        Err(e) => {
            format!(
                "Bank: {}\nStatus: error\nType: bank\nError: {}\n\n",
                repository.current_bank_id(),
                e
            )
        }
    }
}

fn handle_transfer(
    repository: &mut Repository,
    sender: AccountID,
    receiver: AccountID,
    amount: u64,
) -> String {
    match repository.transfer(sender, receiver, amount) {
        Ok(opperation_id) => {
            format!(
                "Bank: {}\nOpID: {}\nStatus: ok\n\n",
                repository.current_bank_id(),
                opperation_id,
            )
        }
        Err(e) => {
            format!(
                "Bank: {}\nStatus: error\nType: bank\nError: {}\n\n",
                repository.current_bank_id(),
                e
            )
        }
    }
}

fn operations_as_string<'a, I: Iterator<Item = &'a Operation>>(operations: I) -> String {
    let operations: Vec<String> = operations.map(|op| op.to_string()).collect();
    if operations.len() == 0 {
        return String::from("no operations yet");
    }
    operations.join("\n")
}

fn handle_list_account_operations(repository: &mut Repository, id: AccountID) -> String {
    let operations = repository.get_account_operations(id);
    format!(
        "Bank: {}\nStatus: ok\nResult:\n{}\n\n",
        repository.current_bank_id(),
        operations_as_string(operations),
    )
}

fn handle_list_all_operations(repository: &mut Repository) -> String {
    let operations = repository.get_all_operations();
    format!(
        "Bank: {}\nStatus: ok\nResult:\n{}\n\n",
        repository.current_bank_id(),
        operations_as_string(operations),
    )
}

fn handle_command(repository: &mut Repository, command: &Command) -> String {
    match *command {
        Command::NewBank => handle_new_bank(repository),
        Command::ChangeBank { id } => handle_change_bank(repository, id),
        Command::RestoreBank { id } => handle_restore_bank(repository, id),
        Command::WhichBank => handle_which_bank(repository),
        Command::RegisterAccount { balance } => handle_register_account(repository, balance),
        Command::GetBalance { id } => handle_get_balance(repository, id),
        Command::Deposit { id, balance } => handle_deposit(repository, id, balance),
        Command::Withdraw { id, balance } => handle_withdraw(repository, id, balance),
        Command::Transfer {
            sender,
            receiver,
            amount,
        } => handle_transfer(repository, sender, receiver, amount),

        Command::ListAccountOperations { id } => handle_list_account_operations(repository, id),
        Command::ListAllOperations => handle_list_all_operations(repository),
        _ => format!(
            "Bank: {}\nStatus: error\nType: repository\nError: unknown command\n\n",
            repository.current_bank_id(),
        ),
    }
}
