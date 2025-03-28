use crate::csv::ToCsv;
use crate::db::TransactionExtras;
use crate::powens::{Account, Transaction};

#[derive(Debug, Clone, PartialEq)]
pub struct TransactionCsv {
    pub id: u64,
    pub date: String,
    pub amount: f64,
    pub name: String,
    pub category: String,
    pub tags: String,
    pub account: String,
    pub notes: String,
}

impl From<&Transaction> for TransactionCsv {
    fn from(t: &Transaction) -> Self {
        TransactionCsv {
            id: t.id,
            date: t.date.clone(),
            amount: t.value,
            name: t.wording.clone(),
            category: String::new(),
            tags: String::new(),
            account: String::new(),
            notes: String::new(),
        }
    }
}

impl TransactionCsv {
    pub fn set_account(&mut self, account: &Account) {
        self.account = account.name.clone();
    }
    
    pub fn set_extras(&mut self, extras: &TransactionExtras) {
        if !extras.categories.is_empty() {
            self.category = extras.categories.last().unwrap().clone();
        }
        
        if !extras.tags.is_empty() {
            self.tags = extras.tags.join("|");
        }
    }
}

impl ToCsv for TransactionCsv {
    fn header_row() -> &'static str {
        "date,amount,name,category,tags,account,notes"
    }

    fn to_csv_row(&self) -> String {
        let TransactionCsv {
            date,
            amount,
            name,
            category,
            tags,
            account,
            notes,
            ..
        } = self;

        let name = Self::format_csv_value(name);
        let notes = Self::format_csv_value(notes);

        format!("{date},{amount:.2},{name},{category},{tags},{account},{notes}")
    }
}
