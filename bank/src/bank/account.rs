use uuid::Uuid;

#[derive(Copy, Clone, PartialEq, Eq, Hash, Debug, Default)]
pub struct AccountID(Uuid);

impl AccountID {
    pub fn new() -> AccountID {
        AccountID(Uuid::new_v4())
    }
}

#[derive(Clone, Copy)]
pub struct Account {
    pub id: AccountID,
    pub balance: u64,
}

impl Account {
    pub fn new(balance: u64) -> Account {
        Account {
            id: AccountID::new(),
            balance,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn new_account_works() {
        let account1 = Account::new(100);
        let account2 = Account::new(200);
        assert_eq!(account1.balance, 100);
        assert_eq!(account2.balance, 200);
    }
}
