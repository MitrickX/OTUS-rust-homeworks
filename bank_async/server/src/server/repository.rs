use crate::bank::account::{Account, AccountID};
use crate::bank::log::{Operation, OperationID};
use crate::bank::{Bank, BankError};

#[derive(Debug, PartialEq)]
pub enum RepositoryError {
    InvalidBankId,
    BankError(BankError),
}

impl std::fmt::Display for RepositoryError {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        match self {
            RepositoryError::InvalidBankId => write!(f, "Invalid bank id"),
            RepositoryError::BankError(e) => write!(f, "Bank error: {}", e),
        }
    }
}

impl std::error::Error for RepositoryError {}

pub type Result<T> = std::result::Result<T, RepositoryError>;

#[derive(Default, Clone)]
pub struct Repository {
    pub banks: Vec<Bank>,
    pub current_bank: usize,
}

impl Repository {
    pub fn current_bank_id(&self) -> usize {
        if self.banks.is_empty() {
            0
        } else {
            self.current_bank + 1
        }
    }

    pub fn new_bank(&mut self) -> usize {
        self.banks.push(Bank::default());
        self.current_bank = self.banks.len() - 1;
        self.current_bank + 1
    }

    pub fn change_bank(&mut self, id: u64) -> Result<()> {
        if id < 1 || id > self.banks.len() as u64 {
            return Err(RepositoryError::InvalidBankId);
        }

        let new_current_bank = (id - 1) as usize;
        self.current_bank = new_current_bank;

        Ok(())
    }

    pub fn restore_bank(&mut self, id: u64) -> Result<()> {
        if id < 1 || id > self.banks.len() as u64 {
            return Err(RepositoryError::InvalidBankId);
        }

        let src_bank = &self.banks[(id - 1) as usize];

        match Bank::restore(src_bank.get_all_operations()) {
            Ok(new_bank) => {
                self.banks.push(new_bank);
                self.current_bank = self.banks.len() - 1;
                Ok(())
            }
            Err(e) => Err(RepositoryError::BankError(e)),
        }
    }

    pub fn register_account(&mut self, balance: u64) -> Result<(AccountID, OperationID)> {
        if self.banks.is_empty() {
            self.new_bank();
        }

        let current_bank = self.current_bank;
        let bank = &mut self.banks[current_bank];
        let account = Account::new(balance);

        match bank.register_account(account) {
            Ok(operation_id) => Ok((account.id, operation_id)),
            Err(e) => Err(RepositoryError::BankError(e)),
        }
    }

    pub fn get_balance(&mut self, id: AccountID) -> Result<u64> {
        if self.banks.is_empty() {
            self.new_bank();
        }

        let bank = &self.banks[self.current_bank];
        bank.get_balance(id).map_err(RepositoryError::BankError)
    }

    pub fn deposit(&mut self, id: AccountID, amount: u64) -> Result<OperationID> {
        if self.banks.is_empty() {
            self.new_bank();
        }

        let bank = &mut self.banks[self.current_bank];
        bank.deposit(id, amount).map_err(RepositoryError::BankError)
    }

    pub fn withdraw(&mut self, id: AccountID, amount: u64) -> Result<OperationID> {
        if self.banks.is_empty() {
            self.new_bank();
        }

        let bank = &mut self.banks[self.current_bank];
        bank.withdraw(id, amount)
            .map_err(RepositoryError::BankError)
    }

    pub fn transfer(
        &mut self,
        sender_id: AccountID,
        receiver_id: AccountID,
        amount: u64,
    ) -> Result<OperationID> {
        if self.banks.is_empty() {
            self.new_bank();
        }

        let bank = &mut self.banks[self.current_bank];
        bank.transfer(sender_id, receiver_id, amount)
            .map_err(RepositoryError::BankError)
    }

    pub fn get_account_operations(&self, id: AccountID) -> impl Iterator<Item = &Operation> {
        let result: Vec<&Operation> = if self.banks.is_empty() {
            Vec::new()
        } else {
            let bank = &self.banks[self.current_bank];
            bank.get_account_operations(id).collect()
        };

        result.into_iter()
    }

    pub fn get_all_operations(&self) -> impl Iterator<Item = &Operation> {
        let result: Vec<&Operation> = if self.banks.is_empty() {
            Vec::new()
        } else {
            let bank = &self.banks[self.current_bank];
            bank.get_all_operations().collect()
        };

        result.into_iter()
    }
}

#[cfg(test)]
mod tests {
    use crate::bank::log::OperationKind;

    use super::*;

    #[test]
    fn new_bank_works() {
        let bank_id = Repository::default().new_bank();
        assert_eq!(bank_id, 1);
    }

    #[test]
    fn current_bank_id_works() {
        let mut repository = Repository::default();
        assert_eq!(repository.current_bank_id(), 0);

        repository.new_bank();
        assert_eq!(repository.current_bank_id(), 1);
    }

    #[test]
    fn change_bank_works() {
        let mut repository = Repository::default();
        repository.new_bank();
        repository.new_bank();
        repository.new_bank();

        assert_eq!(repository.current_bank_id(), 3);

        assert!(repository.change_bank(1).is_ok());
        assert_eq!(repository.current_bank_id(), 1);
        assert!(repository.change_bank(2).is_ok());
        assert_eq!(repository.current_bank_id(), 2);
        assert!(repository.change_bank(3).is_ok());
        assert_eq!(repository.current_bank_id(), 3);

        assert_eq!(
            Err(RepositoryError::InvalidBankId),
            repository.change_bank(0)
        );
        assert_eq!(
            Err(RepositoryError::InvalidBankId),
            repository.change_bank(4)
        );
        assert_eq!(
            Err(RepositoryError::InvalidBankId),
            repository.change_bank(100)
        );

        assert_eq!(repository.current_bank_id(), 3);
    }

    #[test]
    fn register_account_works() {
        let mut repository = Repository::default();
        assert!(repository.register_account(100).is_ok());
        assert!(repository.register_account(0).is_ok());
    }

    #[test]
    fn get_balance_works() {
        let mut repository = Repository::default();
        let (account_id, _) = repository.register_account(100).unwrap();
        assert_eq!(100, repository.get_balance(account_id).unwrap());

        let fake_account = Account::new(10);
        assert!(repository.get_balance(fake_account.id).is_err());
    }

    #[test]
    fn deposit_works() {
        let mut repository = Repository::default();
        let (account_id, _) = repository.register_account(100).unwrap();
        assert!(repository.deposit(account_id, 10).is_ok());
        assert_eq!(110, repository.get_balance(account_id).unwrap());
    }

    #[test]
    fn withdraw_works() {
        let mut repository = Repository::default();
        let (account_id, _) = repository.register_account(100).unwrap();
        assert!(repository.withdraw(account_id, 10).is_ok());
        assert_eq!(90, repository.get_balance(account_id).unwrap());
    }

    #[test]
    fn transfer_works() {
        let mut repository = Repository::default();
        let (sender_id, _) = repository.register_account(100).unwrap();
        let (receiver_id, _) = repository.register_account(100).unwrap();
        assert!(repository.transfer(sender_id, receiver_id, 10).is_ok());
        assert_eq!(90, repository.get_balance(sender_id).unwrap());
        assert_eq!(110, repository.get_balance(receiver_id).unwrap());
    }

    #[test]
    fn get_account_operations_works() {
        let mut repository = Repository::default();
        let (account1_id, _) = repository.register_account(100).unwrap();
        repository.deposit(account1_id, 10).unwrap();
        repository.withdraw(account1_id, 10).unwrap();

        let (account2_id, _) = repository.register_account(50).unwrap();
        repository.transfer(account1_id, account2_id, 10).unwrap();

        let operations: Vec<OperationKind> = repository
            .get_account_operations(account1_id)
            .map(|op| op.kind)
            .collect();

        let expected: Vec<OperationKind> = vec![
            OperationKind::Register {
                id: account1_id,
                balance: 100,
            },
            OperationKind::Deposit {
                id: account1_id,
                amount: 10,
            },
            OperationKind::Withdraw {
                id: account1_id,
                amount: 10,
            },
            OperationKind::Transfer {
                sender_id: account1_id,
                receiver_id: account2_id,
                amount: 10,
            },
        ];
        assert_eq!(operations, expected);
    }

    #[test]
    fn get_all_operations_works() {
        let mut repository = Repository::default();
        let (account1_id, _) = repository.register_account(100).unwrap();
        repository.deposit(account1_id, 10).unwrap();
        repository.withdraw(account1_id, 10).unwrap();

        let (account2_id, _) = repository.register_account(50).unwrap();
        repository.transfer(account1_id, account2_id, 10).unwrap();

        let operations: Vec<OperationKind> =
            repository.get_all_operations().map(|op| op.kind).collect();

        let expected: Vec<OperationKind> = vec![
            OperationKind::Register {
                id: account1_id,
                balance: 100,
            },
            OperationKind::Deposit {
                id: account1_id,
                amount: 10,
            },
            OperationKind::Withdraw {
                id: account1_id,
                amount: 10,
            },
            OperationKind::Register {
                id: account2_id,
                balance: 50,
            },
            OperationKind::Transfer {
                sender_id: account1_id,
                receiver_id: account2_id,
                amount: 10,
            },
        ];
        assert_eq!(operations, expected);
    }

    #[test]
    fn restore_bank_works() {
        let mut repository = Repository::default();
        let (account1_id, _) = repository.register_account(100).unwrap();
        let (account2_id, _) = repository.register_account(50).unwrap();

        repository.deposit(account1_id, 100).unwrap();
        repository.deposit(account2_id, 250).unwrap();
        repository.transfer(account1_id, account2_id, 50).unwrap();
        repository.withdraw(account2_id, 50).unwrap();

        repository.new_bank();
        repository.register_account(150).unwrap();
        repository.register_account(10).unwrap();

        repository.restore_bank(1).unwrap();

        let restored_bank_operations = repository
            .get_all_operations()
            .map(|op| op.to_string())
            .collect::<Vec<_>>();

        assert_eq!(3, repository.current_bank_id());

        repository.change_bank(1).unwrap();

        let original_bank_operations = repository
            .get_all_operations()
            .map(|op| op.to_string())
            .collect::<Vec<_>>();

        assert_eq!(original_bank_operations, restored_bank_operations);
    }
}
