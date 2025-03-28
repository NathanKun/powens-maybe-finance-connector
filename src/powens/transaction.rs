/*!
Structs and Enums related to Powens transactions APIs' responses.
*/

use serde::{Deserialize, Serialize};

/**
Structure representing a Powens Bank Transaction.

See https://docs.powens.com/api-reference/products/data-aggregation/bank-transactions#transaction-object.

Has totally 39 fields during my test, but not all fields are documented.

Removed some fields base on a check of latest 100 transactions:
- 18 fields which are always null.
- id_category which has no official doc and is always 9998.
- state which has no official doc and is always "parsed".
- documents_count which has no official doc and is always 0.
- information field which is always an empty object.
*/
#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
// #[serde(rename_all = "camelCase")]
pub struct Transaction {
    /// ID of the transaction.
    pub id: u64,

    /// ID of the related account.
    // #[serde(rename = "id_account")]
    pub id_account: u64,

    /// Date considered by PFM services. This date can be edited.
    // #[serde(rename = "application_date")]
    pub application_date: String,

    /// Date when the transaction is posted to the account.
    pub date: String,

    /// Value of the transaction.
    pub value: f64,

    /// Full label of the transaction, as seen on the bank.
    // #[serde(rename = "original_wording")]
    pub original_wording: String,

    /// Simplified label of the transaction.
    // #[serde(rename = "simplified_wording")]
    pub simplified_wording: String,

    /// No official doc. Seems to be a list space seperated words stemmed from original_wording.
    /// Example: "carte x www foodles co commerce electronique"
    // #[serde(rename = "stemmed_wording")]
    pub stemmed_wording: String,

    /// Label of the transaction, can be edited.
    /// Seems to be the same of simplified_wording by default.
    pub wording: String,

    /// Date and time when the transaction was seen.
    // #[serde(rename = "date_scraped")]
    pub date_scraped: String,

    /// Date when the transaction order has been given.
    pub rdate: String,

    /// Value date of the transaction. In most cases, equivalent to date.
    pub vdate: Option<String>,

    /// If true, this transaction has not yet been posted to the account.
    pub coming: bool,

    /// If false, PFM services will ignore this transaction.
    pub active: bool,

    /// Last update of the transaction.
    // #[serde(rename = "last_update")]
    pub last_update: String,

    /// Card number associated with the transaction.
    pub card: String,

    /// Type of transaction.
    #[serde(rename = "type")]
    pub type_field: TransactionType,

    /// No official doc. Seems to be a formatted string with the transaction value, currency symbol, and a localized digit seperator.
    /// Example: "-20,00 €"
    // #[serde(rename = "formatted_value")]
    pub formatted_value: String,
}

/**
Response of /2.0/users/me/transactions
*/
#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct TransactionsResponse {
    pub first_date: String,
    pub last_date: String,
    pub transactions: Vec<Transaction>,
    pub total: u64,
    #[serde(rename = "_links")]
    pub links: ResponseLinks,
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ResponseLinks {
    pub prev: Option<String>,
    pub next: Option<ResponseLink>,
    #[serde(rename = "self")]
    pub self_field: Option<ResponseLink>,
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ResponseLink {
    pub href: String,
}

/**
Transaction Type Enum.

See https://docs.powens.com/api-reference/products/data-aggregation/bank-transactions#transactiontype-values
*/
#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum TransactionType {
    /// Transfer
    Transfer,
    /// Order
    Order,
    /// Check
    Check,
    /// Mandatory/voluntary deposits, contributions, money transfers
    Deposit,
    /// Payback
    Payback,
    /// Withdrawal
    Withdrawal,
    /// Loan payment
    LoanRepayment,
    /// Bank fees
    Bank,
    /// Card operation
    Card,
    /// Deferred card operation
    DeferredCard,
    /// Monthly debit of a deferred card
    SummaryCard,
    /// Unknown transaction type
    #[default]
    Unknown,
    /// Market order
    MarketOrder,
    /// Fees regarding a market order
    MarketFee,
    /// Arbitrage
    Arbitrage,
    /// Positive earnings from interests/coupons/dividends
    Profit,
    /// With opposition to a payback, a refund has a negative value
    Refund,
    /// Transfer from the e-commerce account (eg; Stripe) to the bank account
    Payout,
    /// Payment made with a payment method different from card
    Payment,
    /// Differs from bank type because it considers only tax/commission
    Fee,
}
