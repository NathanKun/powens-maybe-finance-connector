use crate::db::{AccountsDb, TransactionExtrasDb, TransactionsDb};
use crate::powens::PowensApi;

#[derive(Clone)]
pub struct AppState {
    pub account_db: AccountsDb,
    pub transaction_db: TransactionsDb,
    pub transaction_extras_db: TransactionExtrasDb,
    pub powens_api: PowensApi,
}
