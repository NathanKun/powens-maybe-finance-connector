//! Struct and methods to call Powens' APIs

use super::{Account, AccountsResponse, Transaction, TransactionsResponse};
use tracing::{debug, error};

#[derive(Clone)]
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

        debug!("Calling Powens API: {}", api);

        let client = reqwest::Client::new();
        let text = client.get(&api).bearer_auth(&token).send().await?.text().await?;
        let json_res = serde_json::from_str::<T>(&text);
        if let Ok(json) = json_res {
            Ok(json)
        } else {
            error!("Failed to decode Powens API response: {:?}", &text);
            Err(Box::new(json_res.err().unwrap()))
        }
    }

    pub async fn get_transactions(&self) -> Result<Vec<Transaction>, Box<dyn std::error::Error>> {
        let resp = self
            .get::<TransactionsResponse>("/2.0/users/me/transactions?limit=1000")
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
