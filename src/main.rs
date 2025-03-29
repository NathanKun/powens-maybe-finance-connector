use axum::{routing::get, Router};
use clokwerk::{Job, Scheduler, TimeUnits};
use powens_maybe_finance_connector::app_state::AppState;
use powens_maybe_finance_connector::db::{AccountsDb, TransactionExtrasDb, TransactionsDb};
use powens_maybe_finance_connector::genai::run_ai_guess_on_all_transactions;
use powens_maybe_finance_connector::handlers::{
    accounts_to_csv_handler, fetch_transactions_from_powens_handler, list_accounts_handler,
    list_transactions_handler, run_fetch_transactions_from_powens_job, transactions_to_csv_handler,
};
use powens_maybe_finance_connector::powens::PowensApi;
use std::time::Duration;
use tokio::signal;
use tower_http::timeout::TimeoutLayer;
use tower_http::trace::TraceLayer;
use tracing::info;
use tracing::error;

#[tokio::main]
async fn main() {
    // initialize tracing
    tracing_subscriber::fmt()
        .with_max_level(tracing::Level::DEBUG)
        .init();

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

    // Create a new scheduler
    let mut scheduler = Scheduler::new();
    {
        let app_state = (&app_state).clone();
        scheduler
            .every(1.day())
            .at(&dotenv::var("SCHEDULER_FETCH_TRANSACTION_AT").unwrap())
            .run(move || {
                let app_state = app_state.clone();
                run_fetch_transactions_from_powens_job(app_state);
            });
    }

    // Run scheduler loop in a spawned task
    tokio::spawn(async move {
        info!("Scheduler started.");
        loop {
            scheduler.run_pending(); // Ensure `run_pending` processes tasks
            tokio::time::sleep(Duration::from_secs(2)).await;
        }
    });

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
        .with_state(app_state)
        .layer((
            TraceLayer::new_for_http(),
            // Graceful shutdown will wait for outstanding requests to complete. Add a timeout so
            // requests don't hang forever.
            TimeoutLayer::new(Duration::from_secs(10)),
        ));

    // run our app with hyper, listening globally on port 3000
    let listener = tokio::net::TcpListener::bind("0.0.0.0:3000").await.unwrap();
    axum::serve(listener, app)
        .with_graceful_shutdown(shutdown_signal())
        .await
        .unwrap();
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

async fn shutdown_signal() {
    let ctrl_c = async {
        signal::ctrl_c()
            .await
            .expect("failed to install Ctrl+C handler");
    };

    #[cfg(unix)]
    let terminate = async {
        signal::unix::signal(signal::unix::SignalKind::terminate())
            .expect("failed to install signal handler")
            .recv()
            .await;
    };

    #[cfg(not(unix))]
    let terminate = std::future::pending::<()>();

    tokio::select! {
        _ = ctrl_c => {
            info!("Received Ctrl+C, shutting down.");
        },
        _ = terminate => {
            info!("Received terminate signal, shutting down.");
        },
    }
}
