use crate::bank::AccountID;
use std::collections::HashMap;
use uuid::Uuid;

#[derive(Debug, PartialEq, Clone, Copy)]
pub enum OperationKind {
    Register {
        id: AccountID,
        balance: u64,
    },
    Deposit {
        id: AccountID,
        amount: u64,
    },
    Withdraw {
        id: AccountID,
        amount: u64,
    },
    Transfer {
        sender_id: AccountID,
        reciever_id: AccountID,
        amount: u64,
    },
}

impl std::fmt::Display for OperationKind {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        match self {
            OperationKind::Register { id, balance } => {
                write!(f, "Register {} {}", id, balance)
            }
            OperationKind::Deposit { id, amount } => {
                write!(f, "Deposit {} {}", id, amount)
            }
            OperationKind::Withdraw { id, amount } => {
                write!(f, "Withdraw {} {}", id, amount)
            }
            OperationKind::Transfer {
                sender_id,
                reciever_id,
                amount,
            } => {
                write!(f, "Transfer {} {} {}", sender_id, reciever_id, amount)
            }
        }
    }
}

#[derive(Debug, PartialEq, Eq, PartialOrd, Ord, Hash, Clone, Copy, Default)]
pub struct OperationID(Uuid);

impl OperationID {
    pub fn new() -> OperationID {
        OperationID(Uuid::new_v4())
    }
}

impl std::fmt::Display for OperationID {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(f, "{}", self.0)
    }
}

#[derive(Debug, PartialEq, Clone, Copy)]
pub struct Operation {
    pub id: OperationID,
    pub kind: OperationKind,
}

impl std::fmt::Display for Operation {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(f, "{}: ({})", self.id, self.kind)
    }
}

#[derive(Debug, Default, PartialEq, Clone)]
pub struct OperationsLog {
    accounts_operations: HashMap<AccountID, Vec<OperationID>>,
    operations_by_id: HashMap<OperationID, usize>,
    operations: Vec<Operation>,
}

impl OperationsLog {
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

    pub fn log_operation(&mut self, operation: Operation) {
        let operation_id = operation.id;
        let operation_kind = operation.kind;

        let operation_idx = self.operations.len();
        self.operations_by_id.insert(operation_id, operation_idx);
        self.operations.push(operation);

        match operation_kind {
            OperationKind::Register { id, .. }
            | OperationKind::Deposit { id, .. }
            | OperationKind::Withdraw { id, .. } => {
                self.log_for_account(id, operation_id);
            }
            OperationKind::Transfer {
                sender_id,
                reciever_id,
                ..
            } => {
                self.log_for_account(sender_id, operation_id);
                self.log_for_account(reciever_id, operation_id);
            }
        }
    }

    pub fn log(&mut self, operation_kind: OperationKind) -> OperationID {
        let operation_id = OperationID::new();
        let operation = Operation {
            id: operation_id,
            kind: operation_kind,
        };

        self.log_operation(operation);

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
