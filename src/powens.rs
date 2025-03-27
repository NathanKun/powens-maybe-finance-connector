mod transaction;
mod account;
mod api;
mod webhook;

pub use self::transaction::*;
pub use self::account::*;
pub use self::api::*;

pub trait HasId {
    fn id(&self) -> u64;
}

impl HasId for Transaction {
    fn id(&self) -> u64 {
        self.id
    }
}

impl HasId for Account {
    fn id(&self) -> u64 {
        self.id
    }
}
