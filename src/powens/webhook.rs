//! Structs related to Powens' webhook requests

use crate::powens::Transaction;

// ACCOUNT_SYNCED
// An ACCOUNT_SYNCED webhook is emitted during a sync after a bank account was processed, including new transactions.
// See https://docs.powens.com/api-reference/products/data-aggregation/bank-accounts#accounts-synced
pub struct AccountSyncedWebhook {
    // {Account} => to be deserialized to Account struct
    transactions: Vec<Transaction>,
}