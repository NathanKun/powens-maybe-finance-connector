use super::db_base::StructFileDb;
use crate::powens::{Account, HasId, Sortable, Transaction};
use serde::{Deserialize, Serialize};
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

/**
Extra data to add to a transaction, to form a complete Maybe-Finance transaction.
*/
#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct TransactionExtras {
    /// to match the Transaction id
    pub id: u64,
    pub categories: Vec<String>,
    pub tags: Vec<String>,
}

impl HasId for TransactionExtras {
    fn id(&self) -> u64 {
        self.id
    }
}

impl Sortable for TransactionExtras {
    fn sortable_value(&self) -> impl Ord {
        self.id.to_string()
    }
}

pub type TransactionExtrasDb = StructFileDb<TransactionExtras>;

impl TransactionExtrasDb {
    pub fn new_transaction_extras_db() -> Result<Self, Box<dyn std::error::Error>> {
        let res = StructFileDb::<TransactionExtras>::new("db/transaction_extras.json".to_string());
        info!("Transaction Extras DB initialized.");
        res
    }
}
