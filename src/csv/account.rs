use crate::csv::ToCsv;
use crate::powens::Account;

#[derive(Debug, Clone, PartialEq)]
pub struct AccountCsv {
    pub id: u64,
    pub account_type: String,
    pub name: String,
    pub balance: f64,
    pub currency: String,
}

impl From<&Account> for AccountCsv {
    fn from(acc: &Account) -> Self {
        AccountCsv {
            id: acc.id,
            account_type: acc.type_field.to_string(),
            name: acc.name.clone(),
            balance: acc.balance,
            currency: acc.currency.id.clone(),
        }
    }
}

impl ToCsv for AccountCsv {
    fn header_row() -> &'static str {
        "Entity type,Name,Balance,Currency"
    }

    fn to_csv_row(&self) -> String {
        let AccountCsv {
            account_type,
            name,
            balance,
            currency,
            ..
        } = self;
        format!("{account_type},{name},{balance:.2},{currency}")
    }
}
