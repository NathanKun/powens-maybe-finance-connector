use super::db_base::StructFileDb;
use crate::powens::{Account, Transaction};
use tracing::info;

pub type AccountsDb = StructFileDb<Account>;

impl AccountsDb {
    pub fn new_account_db() -> Result<Self, Box<dyn std::error::Error>> {
        let res = StructFileDb::<Account>::new("db/accounts.json".to_string());
        info!("Accounts DB initialized.");
        res
    }
}

pub type TransactionsDb = StructFileDb<Transaction>;

impl TransactionsDb {
    pub fn new_transaction_db() -> Result<Self, Box<dyn std::error::Error>> {
        let res = StructFileDb::<Transaction>::new("db/transaction.json".to_string());
        info!("Transactions DB initialized.");
        res
    }
}
