pub mod account;
pub mod log;

use account::*;
use log::*;
use std::collections::HashMap;

#[derive(Debug, PartialEq)]
pub enum BankError {
    NotFound,
    AlreadyExists,
    ZeroAmmount,
    InsufficientFunds,
    TransferToItself,
}

#[derive(Default)]
pub struct Bank {
    accounts: HashMap<AccountID, Account>,
    operations_log: OperationsLog,
}

impl Bank {
    pub fn new() -> Bank {
        Bank {
            accounts: HashMap::new(),
            operations_log: OperationsLog::new(),
        }
    }

    pub fn register_account(&mut self, account: Account) -> Result<OperationID, BankError> {
        let account_id = account.id;
        if self.accounts.contains_key(&account_id) {
            return Err(BankError::AlreadyExists);
        }

        self.accounts.insert(account_id, account);
        let operation_id = self.operations_log.log(OperationKind::Register(account_id));

        Ok(operation_id)
    }

    pub fn get_operation(&self, operation_id: OperationID) -> Option<&Operation> {
        self.operations_log.get(operation_id)
    }

    pub fn get_balance(&self, id: AccountID) -> Result<u64, BankError> {
        match self.accounts.get(&id) {
            Some(account) => Ok(account.balance),
            None => Err(BankError::NotFound),
        }
    }

    fn update_account_balance_by_amount(
        &mut self,
        id: AccountID,
        amount: i64,
    ) -> Result<(), BankError> {
        if amount == 0 {
            return Err(BankError::ZeroAmmount);
        }

        let account = self.accounts.get_mut(&id).ok_or(BankError::NotFound)?;

        let result_balance = account.balance as i64 + amount;
        if result_balance < 0 {
            return Err(BankError::InsufficientFunds);
        }

        account.balance = result_balance as u64;
        Ok(())
    }

    pub fn deposit(&mut self, id: AccountID, amount: u64) -> Result<OperationID, BankError> {
        self.update_account_balance_by_amount(id, amount as i64)?;

        let operation_id = self.operations_log.log(OperationKind::Deposit(id, amount));
        Ok(operation_id)
    }

    pub fn withdraw(&mut self, id: AccountID, amount: u64) -> Result<OperationID, BankError> {
        self.update_account_balance_by_amount(id, -(amount as i64))?;

        let operation_id = self.operations_log.log(OperationKind::Withdraw(id, amount));
        Ok(operation_id)
    }

    pub fn transfer(
        &mut self,
        sender_id: AccountID,
        reciever_id: AccountID,
        amount: u64,
    ) -> Result<OperationID, BankError> {
        if sender_id == reciever_id {
            return Err(BankError::TransferToItself);
        }

        self.update_account_balance_by_amount(sender_id, -(amount as i64))?;
        self.update_account_balance_by_amount(reciever_id, amount as i64)?;

        let operation_id =
            self.operations_log
                .log(OperationKind::Transfer(sender_id, reciever_id, amount));

        Ok(operation_id)
    }

    pub fn get_all_operations(&self) -> impl Iterator<Item = &Operation> {
        self.operations_log.get_all_operations()
    }

    pub fn get_account_operations(
        &self,
        account_id: AccountID,
    ) -> impl Iterator<Item = &Operation> {
        self.operations_log.get_account_operations(account_id)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn register_account_works() {
        let mut bank = Bank::new();
        let account1 = Account::new(100);
        let account2 = Account::new(200);

        let account1_id = account1.id;
        let account2_id = account2.id;

        let operation1_id = bank.register_account(account1).unwrap();
        let operation2_id = bank.register_account(account2).unwrap();

        assert_ne!(operation1_id, operation2_id);

        assert_eq!(
            bank.get_operation(operation1_id),
            Some(&Operation {
                id: operation1_id,
                kind: OperationKind::Register(account1_id)
            })
        );

        assert_eq!(
            bank.get_operation(operation2_id),
            Some(&Operation {
                id: operation2_id,
                kind: OperationKind::Register(account2_id)
            })
        );

        let account3 = account1;
        assert_eq!(
            bank.register_account(account3),
            Err(BankError::AlreadyExists)
        );
    }

    #[test]
    fn get_balance_works() {
        let mut bank = Bank::new();

        let account1 = Account::new(100);
        let account2 = Account::new(200);
        let account1_id = account1.id;
        let account2_id = account2.id;
        let account_undifned_id = AccountID::new();

        bank.register_account(account1).unwrap();
        bank.register_account(account2).unwrap();

        assert_eq!(bank.get_balance(account1_id), Ok(100));
        assert_eq!(bank.get_balance(account2_id), Ok(200));
        assert_eq!(
            bank.get_balance(account_undifned_id),
            Err(BankError::NotFound)
        );
    }

    #[test]
    fn deposit_works() {
        let mut bank = Bank::new();
        let account = Account::new(100);
        let account_id = account.id;

        bank.register_account(account).unwrap();

        assert_eq!(bank.deposit(account_id, 0), Err(BankError::ZeroAmmount));

        let operation_id = bank.deposit(account_id, 50).unwrap();
        assert_eq!(
            bank.get_operation(operation_id).unwrap().kind,
            OperationKind::Deposit(account_id, 50)
        );
        assert_eq!(bank.get_balance(account_id).unwrap(), 150);

        let account_undifned_id = AccountID::new();
        assert_eq!(
            bank.deposit(account_undifned_id, 40),
            Err(BankError::NotFound)
        );
    }

    #[test]
    fn withdraw_works() {
        let mut bank = Bank::new();
        let account = Account::new(100);
        let account_id = account.id;
        bank.register_account(account).unwrap();

        assert_eq!(bank.withdraw(account_id, 0), Err(BankError::ZeroAmmount));
        assert_eq!(
            bank.withdraw(account_id, 200),
            Err(BankError::InsufficientFunds)
        );

        let operation_id = bank.withdraw(account_id, 50).unwrap();
        assert_eq!(
            bank.get_operation(operation_id).unwrap().kind,
            OperationKind::Withdraw(account_id, 50)
        );
        assert_eq!(bank.get_balance(account_id).unwrap(), 50);

        let account_undifned_id = AccountID::new();
        assert_eq!(
            bank.withdraw(account_undifned_id, 10),
            Err(BankError::NotFound)
        )
    }

    #[test]
    fn transfer_works() {
        let mut bank = Bank::new();
        let sender = Account::new(100);
        let reciever = Account::new(200);
        let sender_id = sender.id;
        let reciever_id = reciever.id;

        bank.register_account(sender).unwrap();
        bank.register_account(reciever).unwrap();

        assert_eq!(
            bank.transfer(sender_id, reciever_id, 0),
            Err(BankError::ZeroAmmount)
        );
        assert_eq!(
            bank.transfer(sender_id, reciever_id, 1000),
            Err(BankError::InsufficientFunds)
        );
        assert_eq!(
            bank.transfer(sender_id, sender_id, 50),
            Err(BankError::TransferToItself)
        );

        let operation_id = bank.transfer(sender_id, reciever_id, 50).unwrap();
        assert_eq!(
            bank.get_operation(operation_id).unwrap().kind,
            OperationKind::Transfer(sender_id, reciever_id, 50),
        );

        assert_eq!(bank.get_balance(sender_id).unwrap(), 50);
        assert_eq!(bank.get_balance(reciever_id).unwrap(), 250);
    }

    #[test]
    fn get_all_operations_works() {
        let mut bank = Bank::new();

        let account1 = Account::new(100);
        let account2 = Account::new(200);
        let account3 = Account::new(300);

        let account1_id = account1.id;
        let account2_id = account2.id;
        let account3_id = account3.id;

        bank.register_account(account1).unwrap();
        bank.register_account(account2).unwrap();
        bank.register_account(account3).unwrap();

        bank.deposit(account1_id, 50).unwrap();
        bank.withdraw(account2_id, 50).unwrap();
        bank.transfer(account3_id, account2_id, 10).unwrap();

        let operations = bank
            .get_all_operations()
            .map(|operation| operation.kind)
            .collect::<Vec<OperationKind>>();

        let expected_operations = vec![
            OperationKind::Register(account1_id),
            OperationKind::Register(account2_id),
            OperationKind::Register(account3_id),
            OperationKind::Deposit(account1_id, 50),
            OperationKind::Withdraw(account2_id, 50),
            OperationKind::Transfer(account3_id, account2_id, 10),
        ];

        assert_eq!(expected_operations, operations);
    }

    #[test]
    fn get_account_operations_works() {
        let mut bank = Bank::new();

        let account1 = Account::new(100);
        let account2 = Account::new(200);
        let account3 = Account::new(300);

        let account1_id = account1.id;
        let account2_id = account2.id;
        let account3_id = account3.id;

        bank.register_account(account1).unwrap();
        bank.register_account(account2).unwrap();
        bank.register_account(account3).unwrap();

        bank.deposit(account1_id, 50).unwrap();
        bank.withdraw(account2_id, 50).unwrap();
        bank.transfer(account3_id, account2_id, 20).unwrap();
        bank.deposit(account1_id, 150).unwrap();
        bank.withdraw(account1_id, 10).unwrap();
        bank.transfer(account1_id, account2_id, 10).unwrap();

        let account1_operations = bank
            .get_account_operations(account1_id)
            .map(|operation| operation.kind)
            .collect::<Vec<OperationKind>>();

        let account1_expected_operations = vec![
            OperationKind::Register(account1_id),
            OperationKind::Deposit(account1_id, 50),
            OperationKind::Deposit(account1_id, 150),
            OperationKind::Withdraw(account1_id, 10),
            OperationKind::Transfer(account1_id, account2_id, 10),
        ];

        assert_eq!(account1_expected_operations, account1_operations);

        let account2_operations = bank
            .get_account_operations(account2_id)
            .map(|operation| operation.kind)
            .collect::<Vec<OperationKind>>();

        let account2_expected_operations = vec![
            OperationKind::Register(account2_id),
            OperationKind::Withdraw(account2_id, 50),
            OperationKind::Transfer(account3_id, account2_id, 20),
            OperationKind::Transfer(account1_id, account2_id, 10),
        ];

        assert_eq!(account2_expected_operations, account2_operations);

        let account3_operations = bank
            .get_account_operations(account3_id)
            .map(|operation| operation.kind)
            .collect::<Vec<OperationKind>>();

        let account3_expected_operations = vec![
            OperationKind::Register(account3_id),
            OperationKind::Transfer(account3_id, account2_id, 20),
        ];

        assert_eq!(account3_expected_operations, account3_operations);
    }
}
