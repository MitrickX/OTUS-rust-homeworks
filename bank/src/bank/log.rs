use crate::bank::AccountID;
use std::collections::HashMap;
use uuid::Uuid;

#[derive(Debug, PartialEq, Clone, Copy)]
pub enum OperationKind {
    Register(AccountID),                 // account_id
    Deposit(AccountID, u64),             // account_id, amount
    Withdraw(AccountID, u64),            // account_id, amount
    Transfer(AccountID, AccountID, u64), // sender_id, receiver_id, amount
}

#[derive(Debug, PartialEq, Eq, PartialOrd, Ord, Hash, Clone, Copy, Default)]
pub struct OperationID(Uuid);

impl OperationID {
    pub fn new() -> OperationID {
        OperationID(Uuid::new_v4())
    }
}

#[derive(Debug, PartialEq)]
pub struct Operation {
    pub id: OperationID,
    pub kind: OperationKind,
}

#[derive(Default)]
pub struct OperationsLog {
    accounts_operations: HashMap<AccountID, Vec<OperationID>>,
    operations_by_id: HashMap<OperationID, usize>,
    operations: Vec<Operation>,
}

impl OperationsLog {
    pub fn new() -> OperationsLog {
        OperationsLog {
            accounts_operations: HashMap::new(),
            operations_by_id: HashMap::new(),
            operations: Vec::new(),
        }
    }

    pub fn get(&self, operation_id: OperationID) -> Option<&Operation> {
        self.operations_by_id
            .get(&operation_id)
            .map(|idx| &self.operations[*idx])
    }

    fn log_for_account(&mut self, account_id: AccountID, operation_id: OperationID) {
        self.accounts_operations
            .entry(account_id)
            .or_default()
            .push(operation_id);
    }

    pub fn log(&mut self, operation_kind: OperationKind) -> OperationID {
        let operation_id = OperationID::new();
        let operation = Operation {
            id: operation_id,
            kind: operation_kind,
        };

        let operation_idx = self.operations.len();
        self.operations_by_id.insert(operation_id, operation_idx);
        self.operations.push(operation);

        match operation_kind {
            OperationKind::Register(account_id)
            | OperationKind::Deposit(account_id, _)
            | OperationKind::Withdraw(account_id, _) => {
                self.log_for_account(account_id, operation_id);
            }
            OperationKind::Transfer(sender_id, reciever_id, _) => {
                self.log_for_account(sender_id, operation_id);
                self.log_for_account(reciever_id, operation_id);
            }
        }

        operation_id
    }

    pub fn get_all_operations(&self) -> impl Iterator<Item = &Operation> {
        self.operations.iter()
    }

    pub fn get_account_operations(
        &self,
        account_id: AccountID,
    ) -> impl Iterator<Item = &Operation> {
        self.accounts_operations
            .get(&account_id)
            .map_or(Default::default(), |operation_ids| operation_ids.iter())
            .map(|operation_id| self.get(*operation_id).unwrap())
    }
}
