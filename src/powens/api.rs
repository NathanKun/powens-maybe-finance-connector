//! Struct and methods to call Powens' APIs

use chrono::{DateTime, Utc};
use super::{Account, AccountsResponse, Transaction, TransactionsResponse};
use tracing::{debug, error};

pub const POWENS_DATETIME_FORMAT: &str = "%Y-%m-%d %H:%M:%S";

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

    pub async fn get_transactions(&self, latest_last_update: Option<DateTime<Utc>>) -> Result<Vec<Transaction>, Box<dyn std::error::Error>> {
        let last_update: String = if let Some(latest_last_update) = latest_last_update {
            let value = latest_last_update.format(POWENS_DATETIME_FORMAT).to_string();
            format!("&last_update={}", value)
        } else {
            String::new()
        };
        
        let resp = self
            .get::<TransactionsResponse>(&format!("/2.0/users/me/transactions?limit=1000{last_update}"))
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
