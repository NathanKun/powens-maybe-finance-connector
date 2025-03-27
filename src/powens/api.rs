//! Struct and methods to call Powens' APIs

use super::{Account, AccountsResponse, Transaction, TransactionsResponse};
use tracing::trace;

pub struct PowensApi {
    token: String,
    domain: String,
}

impl PowensApi {
    pub fn new() -> Result<Self, Box<dyn std::error::Error>> {
        Ok(Self {
            token: dotenv::var("POWENS_TOKEN")?,
            domain: dotenv::var("POWENS_APP_DOMAIN")?,
        })
    }

    async fn get<T>(&self, path: &str) -> Result<T, Box<dyn std::error::Error>>
    where
        T: for<'de> serde::Deserialize<'de>,
    {
        let api = format!("{}{}", self.domain, path);
        let token = &self.token;

        trace!("Calling Powens API: {}", api);

        let client = reqwest::Client::new();
        Ok(client
            .get(&api)
            .bearer_auth(&token)
            .send()
            .await?
            .json::<T>()
            .await?)
    }

    pub async fn get_transactions(&self) -> Result<Vec<Transaction>, Box<dyn std::error::Error>> {
        let resp = self
            .get::<TransactionsResponse>("/2.0/users/me/transactions?limit=100")
            .await?;
        Ok(resp.transactions)
    }

    pub async fn get_accounts(&self) -> Result<Vec<Account>, Box<dyn std::error::Error>> {
        let resp = self
            .get::<AccountsResponse>("/2.0/users/me/accounts")
            .await?;
        Ok(resp.accounts)
    }
}
