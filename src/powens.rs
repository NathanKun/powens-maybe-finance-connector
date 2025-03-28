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

pub trait Sortable {
    fn sortable_value(&self) -> impl Ord;
}

impl Sortable for Account {
    fn sortable_value(&self) -> impl Ord {
        self.id
    }
}

impl Sortable for Transaction {
    fn sortable_value(&self) -> impl Ord {
        self.date.clone()
    }
}
