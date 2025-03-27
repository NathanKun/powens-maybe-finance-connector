use axum::{routing::get, Router};
use powens_maybe_finance_connector::db::{AccountsDb, TransactionsDb};
use powens_maybe_finance_connector::powens::PowensApi;
use std::collections::HashMap;
use tracing::info;

#[tokio::main]
async fn main() {
    // initialize tracing
    tracing_subscriber::fmt::init();

    // init file DBs
    let account_db: AccountsDb = match AccountsDb::new_account_db() {
        Ok(db) => db,
        Err(e) => {
            eprintln!("Error creating AccountDb: {}", e);
            return;
        }
    };

    let transaction_db: TransactionsDb = match TransactionsDb::new_transaction_db() {
        Ok(db) => db,
        Err(e) => {
            eprintln!("Error creating TransactionDb: {}", e);
            return;
        }
    };

    // init APIs caller
    let powens_api = match PowensApi::new() {
        Ok(api) => api,
        Err(e) => {
            eprintln!("Error creating PowensApi: {}", e);
            return;
        }
    };

    // check and get initial data from powens if needed
    if let Err(e) = get_initial_data_if_empty(&powens_api, &account_db, &transaction_db).await {
        eprintln!("Error initializing data: {}", e);
        return;
    }

    // build our application with a route
    let app = Router::new()
        // `GET /` goes to `root`
        .route("/", get(root))
        .route(
            "/transactions",
            get(async move || serde_json::to_string_pretty(&transaction_db.data()).unwrap()),
        )
        .route(
            "/accounts",
            get(async move || serde_json::to_string_pretty(&account_db.data()).unwrap()),
        );

    // run our app with hyper, listening globally on port 3000
    let listener = tokio::net::TcpListener::bind("0.0.0.0:3000").await.unwrap();
    axum::serve(listener, app).await.unwrap();
}

async fn root() -> String {
    test_reqwest().await.unwrap()
}

async fn test_reqwest() -> Result<String, Box<dyn std::error::Error>> {
    let resp = reqwest::get("https://httpbin.org/ip")
        .await?
        .json::<HashMap<String, String>>()
        .await?;
    println!("{resp:#?}");
    Ok(resp["origin"].to_string())
}

async fn get_initial_data_if_empty(
    powens_api: &PowensApi,
    account_db: &AccountsDb,
    transaction_db: &TransactionsDb,
) -> Result<(), Box<dyn std::error::Error>> {
    if account_db.is_data_empty() {
        info!("No data found in account DB, getting data from Powens.");
        let accounts = powens_api.get_accounts().await?;
        account_db.save(accounts)?;
    }

    if transaction_db.is_data_empty() {
        info!("No data found in transaction DB, getting data from Powens.");
        let transactions = powens_api.get_transactions().await?;
        transaction_db.save(transactions)?;
    }

    Ok(())
}
