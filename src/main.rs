use axum::extract::State;
use axum::{routing::get, Router};
use chrono::{DateTime, NaiveDateTime, Utc};
use powens_maybe_finance_connector::app_state::AppState;
use powens_maybe_finance_connector::csv::{AccountCsv, TransactionCsv, VecToCsv};
use powens_maybe_finance_connector::db::{
    AccountsDb, TransactionExtras, TransactionExtrasDb, TransactionsDb,
};
use powens_maybe_finance_connector::genai::{
    ai_guess_transaction_categories, run_ai_guess_on_all_transactions,
};
use powens_maybe_finance_connector::handlers::{
    accounts_to_csv_handler, fetch_transactions_from_powens_handler, list_accounts_handler,
    list_transactions_handler, transactions_to_csv_handler,
};
use powens_maybe_finance_connector::powens::{PowensApi, POWENS_DATETIME_FORMAT};
use std::time::{SystemTime, UNIX_EPOCH};
use tracing::info;
use tracing::log::error;

#[tokio::main]
async fn main() {
    // initialize tracing
    tracing_subscriber::fmt::init();

    // init file DBs
    let account_db: AccountsDb = match AccountsDb::new_account_db() {
        Ok(db) => db,
        Err(e) => {
            error!("Error creating AccountDb: {:#?}", e);
            return;
        }
    };

    let transaction_db: TransactionsDb = match TransactionsDb::new_transaction_db() {
        Ok(db) => db,
        Err(e) => {
            error!("Error creating TransactionDb: {:#?}", e);
            return;
        }
    };

    let transaction_extras_db: TransactionExtrasDb =
        match TransactionExtrasDb::new_transaction_extras_db() {
            Ok(db) => db,
            Err(e) => {
                error!("Error creating TransactionExtrasDb: {:#?}", e);
                return;
            }
        };

    // init Powens APIs caller
    let powens_api = match PowensApi::new() {
        Ok(api) => api,
        Err(e) => {
            error!("Error creating PowensApi: {:#?}", e);
            return;
        }
    };

    // App State
    let app_state = AppState {
        account_db,
        transaction_db,
        transaction_extras_db,
        powens_api,
    };

    // check and get initial data from powens if needed
    if let Err(e) = get_initial_powens_data_if_empty(&app_state).await {
        error!("Error initializing data: {:#?}", e);
        return;
    }

    // do AI guessing to generate transaction extras data on powens transactions
    // (only for those have no extras data)
    // run in a seperated thread
    run_ai_guess_job(&app_state);

    // build our application with a route
    let app = Router::new()
        // `GET /` goes to `root`
        .route("/", get(root))
        .route("/transactions", get(list_transactions_handler))
        .route("/transactions/csv", get(transactions_to_csv_handler))
        .route(
            "/transactions/fetch",
            get(fetch_transactions_from_powens_handler),
        )
        .route("/accounts", get(list_accounts_handler))
        .route("/accounts/csv", get(accounts_to_csv_handler))
        .with_state(app_state);

    // run our app with hyper, listening globally on port 3000
    let listener = tokio::net::TcpListener::bind("0.0.0.0:3000").await.unwrap();
    axum::serve(listener, app).await.unwrap();
}

async fn root() -> String {
    "ok".to_string()
}

fn run_ai_guess_job(app_state: &AppState) {
    let app_state = app_state.clone();
    tokio::spawn(async move {
        if let Err(e) = run_ai_guess_on_all_transactions(app_state).await {
            error!("Error running AI guessing: {:#?}", e);
        }
    });
}

async fn get_initial_powens_data_if_empty(
    app_state: &AppState,
) -> Result<(), Box<dyn std::error::Error>> {
    if app_state.account_db.is_data_empty() {
        info!("No data found in account DB, getting data from Powens.");
        let accounts = app_state.powens_api.get_accounts().await?;
        app_state.account_db.save(accounts)?;
    }

    if app_state.transaction_db.is_data_empty() {
        info!("No data found in transaction DB, getting data from Powens.");
        let transactions = app_state.powens_api.get_transactions(None).await?;
        app_state.transaction_db.save(transactions)?;
    }

    info!("Powens data initialized.");

    Ok(())
}
