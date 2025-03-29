use axum::extract::State;
use axum::{Router, routing::get};
use chrono::{DateTime, NaiveDateTime, Utc};
use powens_maybe_finance_connector::csv::{AccountCsv, TransactionCsv, VecToCsv};
use powens_maybe_finance_connector::db::{
    AccountsDb, TransactionExtras, TransactionExtrasDb, TransactionsDb,
};
use powens_maybe_finance_connector::genai::ai_guess_transaction_categories;
use powens_maybe_finance_connector::powens::{POWENS_DATETIME_FORMAT, PowensApi};
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
        .route(
            "/test-ai-guess-transaction-categories",
            get(ai_guess_transaction_categories_handler),
        )
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

async fn fetch_transactions_from_powens_handler(State(app_state): State<AppState>) -> String {
    // TODO: use last_update param. search in DB to find the latest last_update, pass it in API param.
    tokio::spawn(async move {
        let mut latest_last_update: Option<DateTime<Utc>> = None;

        let existing_transactions = app_state.transaction_db.data();
        if !existing_transactions.is_empty() {
            for transaction in existing_transactions {
                let last_update =
                    NaiveDateTime::parse_from_str(&transaction.last_update, POWENS_DATETIME_FORMAT);
                if let Ok(last_update) = last_update {
                    let last_update: DateTime<Utc> = last_update.and_utc();
                    if let Some(latest_last_update_inner) = latest_last_update {
                        if last_update > latest_last_update_inner {
                            latest_last_update = Some(last_update);
                        }
                    } else {
                        latest_last_update = Some(last_update);
                    }
                }
            }
        }

        if let Some(latest_last_update) = latest_last_update {
            info!("last_update: {}", latest_last_update);
        } else {
            info!("No last_update, fetching last 1000 transactions.")
        }

        info!("Starting job to fetch transactions from Powens.");
        if let Ok(transactions) = app_state
            .powens_api
            .get_transactions(latest_last_update)
            .await
        {
            info!("Fetched {} transactions from Powens.", transactions.len());
            for transaction in transactions {
                if let Err(e) = app_state.transaction_db.upsert(transaction) {
                    error!("Error saving transaction: {:#?}", e);
                }
            }
            info!("Transactions saved.");
        }

        // run ai guessing
        if let Err(e) = run_ai_guess_on_all_transactions(app_state).await {
            error!("Error running AI guessing: {:#?}", e);
        }

        info!("Job finished.");
    });

    "Job started".to_string()
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

async fn run_ai_guess_on_all_transactions(
    app_state: AppState,
) -> Result<(), Box<dyn std::error::Error>> {
    let mut transactions = app_state.transaction_db.data(); // this is a clone of Vec<Transaction> at this moment

    // skip if transaction_extras exist & has categories
    transactions.retain(|t| {
        let extras = app_state.transaction_extras_db.find_by_id(t.id);
        extras.is_none() || extras.unwrap().categories.is_empty()
    });

    info!(
        "Running AI guessing on {} transactions.",
        transactions.len()
    );

    for transaction in transactions {
        // do ai guessing
        let categories = ai_guess_transaction_categories(&transaction).await?;

        // create new transaction_extras and save
        let transaction_extras: TransactionExtras = TransactionExtras {
            id: transaction.id,
            categories,
            tags: vec![],
        };

        app_state.transaction_extras_db.upsert(transaction_extras)?;

        // wait 10s to avoid rate limit (gemma 3 is only in free tier and has very strict rate limit)
        tokio::time::sleep(std::time::Duration::from_secs(10)).await;
    }

    info!("AI guessing finished.");
    Ok(())
}

async fn ai_guess_transaction_categories_handler(State(app_state): State<AppState>) -> String {
    let vec = app_state.transaction_db.data();

    // get random transaction in db
    let len = vec.len();
    if len == 0 {
        return String::new();
    }

    // Generate a random number based on time
    let now = SystemTime::now();
    let since_epoch = now.duration_since(UNIX_EPOCH).expect("Time went backwards");
    let nanos = since_epoch.as_micros();

    // Use modulo operator to generate an index within the vector's bounds
    let random_index = (nanos % len as u128) as usize;

    let transaction = vec.get(random_index).unwrap();

    let categories = ai_guess_transaction_categories(&transaction).await.unwrap();

    format!(
        "{}\n{}\n{}\n{:?}",
        transaction.value, transaction.original_wording, transaction.simplified_wording, categories
    )
}

async fn list_transactions_handler(State(app_state): State<AppState>) -> String {
    serde_json::to_string_pretty(&app_state.transaction_db.data()).unwrap()
}

async fn transactions_to_csv_handler(State(app_state): State<AppState>) -> String {
    let account_db = &app_state.account_db;
    let transactions_csv: Vec<TransactionCsv> = (&app_state.transaction_db)
        .data()
        .iter()
        .map(|it| {
            let mut transaction_csv: TransactionCsv = it.into();

            let account = account_db.find_by_id(it.id_account).unwrap();
            transaction_csv.set_account(&account);

            if let Some(extras) = app_state.transaction_extras_db.find_by_id(it.id) {
                transaction_csv.set_extras(&extras);
            }

            transaction_csv
        })
        .collect();
    transactions_csv.to_csv()
}

async fn list_accounts_handler(State(app_state): State<AppState>) -> String {
    serde_json::to_string_pretty(&app_state.account_db.data()).unwrap()
}

async fn accounts_to_csv_handler(State(app_state): State<AppState>) -> String {
    let accounts_csv: Vec<AccountCsv> = (&app_state.account_db)
        .data()
        .iter()
        .map(|it| it.into())
        .collect();
    accounts_csv.to_csv()
}

#[derive(Clone)]
struct AppState {
    account_db: AccountsDb,
    transaction_db: TransactionsDb,
    transaction_extras_db: TransactionExtrasDb,
    powens_api: PowensApi,
}
