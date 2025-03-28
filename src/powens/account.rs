/*!
Structs and Enums related to Powens accounts APIs' responses.
*/

use serde::{Deserialize, Serialize};
use serde_json::Value;

/**
Structure representing a Powens Bank Account.

See https://docs.powens.com/api-reference/products/data-aggregation/bank-accounts#bankaccount-object

Has totally 30 fields during my test, but not all fields are documented.

Removed some fields base on a check of latest 100 transactions:
- fields which are always null.
- information field which is always an empty object.
*/
#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Account {
    /// ID of the bank account.
    pub id: u64,
    /// ID of the related connection.
    pub id_connection: u64,
    /// ID of the related user.
    pub id_user: u64,
    /// ID of the related connection source.
    pub id_source: u64,
    /// Account number.
    pub number: String,
    /// not documented
    pub webid: String,
    /// Original name of the account, as seen on the bank.
    pub original_name: String,
    /// Balance of the account.
    pub balance: f64,
    /// Amount of coming operations not yet debited.
    pub coming: Option<f64>,
    /// Whether the bank account should be presented.
    pub display: bool,
    /// Last successful update of the account.
    pub last_update: String,
    /// DateTime , If set, this account is not found on the website anymore.
    pub deleted: Option<String>,
    /// DateTime , If set, this account has been disabled by user and will not be synchronized anymore.
    pub disabled: Option<String>,
    /// Account IBAN.
    pub iban: String,
    /// Account currency.
    pub currency: Currency,
    /// ID of the account type.
    pub id_type: u64,
    /// This account has been bookmarked by user.
    pub bookmarked: u64,
    /// The name of the account.
    pub name: String,
    /// DateTime , if the last update has failed, the error code.
    pub error: Option<String>,
    /// Account usage. If not overridden, the value of original_usage is returned.
    pub usage: BankAccountUsage,
    /// not documented, but should be the IBAN BIC
    pub bic: String,
    /// not documented, should be the balance after deducting not yet debited coming operations
    pub coming_balance: f64,
    /// not documented, should be a formatted balance value, with currency symbol and localized digit separator, like: "123,45 €"
    pub formatted_balance: String,
    /// Technical code of the account type.
    #[serde(rename = "type")]
    pub type_field: AccountType ,
}

/**
Response of /2.0/users/me/accounts
*/
#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct AccountsResponse {
    pub balance: f64,
    pub balances: Balances,
    pub coming_balances: Balances,
    pub accounts: Vec<Account>,
    pub total: i64,
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum BankAccountUsage {
    /// Private account.
    PRIV,
    /// Professional account.
    ORGA,
    /// No usage detail.
    #[default]
    NULL,
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize, strum::Display)]
#[serde(rename_all = "snake_case")]
pub enum AccountType {
    /// Article 83.
    Article83,
    /// Capitalization contract.
    Capitalisation,
    /// Card.
    Card,
    /// Checking account.
    Checking,
    /// Crowdlending.
    Crowdlending,
    /// Deposit account.
    Deposit,
    /// (Deprecated) Joint account.
    Joint,
    /// Sustainable and solidarity development savings (Livrets de développement durable et solidaire).
    Ldds,
    /// Life insurance account.
    Lifeinsurance,
    /// Loan.
    Loan,
    /// Madelin retirement contract.
    Madelin,
    /// Market account.
    Market,
    /// Shared savings plan (Plan d'Épargne en Actions).
    Pea,
    /// Company savings plan (Plan d'Épargne Entreprise).
    Pee,
    /// Retirement savings plan (Plan d'Épargne Retraite).
    Per,
    /// Group retirement savings plan (Plan d'Épargne pour la Retraite Collectif).
    Perco,
    /// Popular retirement savings plan (Plan d'Épargne Retraite Populaire).
    Perp,
    /// Real estate placement.
    RealEstate,
    /// Special profit-sharing reserve (Réserve Spéciale de Participation).
    Rsp,
    /// Savings account.
    Savings,
    /// Unknown account type.
    #[default]
    Unknown,
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Currency {
    /// ex: EUR
    pub id: String,
    /// ex: €
    pub symbol: String,
    pub prefix: bool,
    pub crypto: bool,
    pub precision: i64,
    pub marketcap: Value,
    pub datetime: Value,
    /// ex: Euro
    pub name: String,
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Balances {
    #[serde(rename = "EUR")]
    pub eur: f64,
}
