use axum::{Router, routing::get};
use powens_maybe_finance_connector::csv::{AccountCsv, TransactionCsv, VecToCsv};
use powens_maybe_finance_connector::db::{AccountsDb, TransactionExtras, TransactionExtrasDb, TransactionsDb};
use powens_maybe_finance_connector::powens::PowensApi;
use std::collections::HashMap;
use std::time::{SystemTime, UNIX_EPOCH};
use axum::extract::State;
use tracing::info;
use powens_maybe_finance_connector::genai::ai_guess_transaction_categories;

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

    let transaction_extras_db: TransactionExtrasDb = match TransactionExtrasDb::new_transaction_extras_db() {
        Ok(db) => db,
        Err(e) => {
            eprintln!("Error creating TransactionExtrasDb: {}", e);
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

    // App State
    let app_state = AppState {
        account_db,
        transaction_db,
        transaction_extras_db,
        powens_api,
    };

    // check and get initial data from powens if needed
    if let Err(e) = get_initial_powens_data_if_empty(&app_state).await {
        eprintln!("Error initializing data: {}", e);
        return;
    }

    // build our application with a route
    let app = Router::new()
        // `GET /` goes to `root`
        .route("/", get(root))
        .route(
            "/transactions",
            get(list_transactions_handler),
        )
        .route(
            "/transactions/csv",
            get(transactions_to_csv_handler),
        )
        .route(
            "/accounts",
            get(list_accounts_handler),
        )
        .route(
            "/accounts/csv",
            get(accounts_to_csv_handler),
        )
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

async fn get_initial_powens_data_if_empty(
    app_state: &AppState
) -> Result<(), Box<dyn std::error::Error>> {
    if app_state.account_db.is_data_empty() {
        info!("No data found in account DB, getting data from Powens.");
        let accounts = app_state.powens_api.get_accounts().await?;
        app_state.account_db.save(accounts)?;
    }

    if app_state.transaction_db.is_data_empty() {
        info!("No data found in transaction DB, getting data from Powens.");
        let transactions = app_state.powens_api.get_transactions().await?;
        app_state.transaction_db.save(transactions)?;
    }

    Ok(())
}

async fn run_ai_guess_on_all_transactions(transactions_db: &TransactionsDb, transaction_extras_db: &TransactionExtrasDb) {
    let transactions = transactions_db.data(); // this is a clone of Vec<Transaction> at this moment
    for transaction in transactions {
        // skip if transaction_extras already exist
        if transaction_extras_db.find_by_id(transaction.id).is_some() {
            continue;
        }
        
        // do ai guessing
        let categories = ai_guess_transaction_categories(&transaction).await.unwrap();
        
        // create new transaction_extras and save
        let transaction_extras: TransactionExtras = TransactionExtras {
            id: transaction.id,
            categories,
            tags: vec![],
        };
        
        transaction_extras_db.upsert(transaction_extras).unwrap();
        
        // wait 10s to avoid rate limit (gemma 3 is only in free tier and has very strict rate limit)
        tokio::time::sleep(std::time::Duration::from_secs(10)).await;
    }
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

    format!("{}\n{}\n{}\n{:?}", transaction.value, transaction.original_wording, transaction.simplified_wording, categories)
}

async fn list_transactions_handler(State(app_state): State<AppState>) -> String {
    serde_json::to_string_pretty(&app_state.transaction_db.data()).unwrap()
}

async fn transactions_to_csv_handler(State(app_state): State<AppState>) -> String {
    let account_db = &app_state.account_db;
    let transactions_csv: Vec<TransactionCsv> =
        (&app_state.transaction_db).data().iter()
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
    let accounts_csv: Vec<AccountCsv> =
        (&app_state.account_db).data().iter().map(|it| it.into()).collect();
    accounts_csv.to_csv()
}

#[derive(Clone)]
struct AppState {
    account_db: AccountsDb,
    transaction_db: TransactionsDb,
    transaction_extras_db: TransactionExtrasDb,
    powens_api: PowensApi,
}
