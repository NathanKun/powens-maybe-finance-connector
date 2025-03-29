use axum::extract::State;
use chrono::{DateTime, NaiveDateTime, Utc};
use tracing::info;
use tracing::error;
use crate::app_state::AppState;
use crate::csv::{TransactionCsv, VecToCsv};
use crate::genai::run_ai_guess_on_all_transactions;
use crate::powens::POWENS_DATETIME_FORMAT;

pub async fn fetch_transactions_from_powens_handler(State(app_state): State<AppState>) -> String {
    run_fetch_transactions_from_powens_job(app_state);

    "Job started".to_string()
}

pub fn run_fetch_transactions_from_powens_job(app_state: AppState) {
    tokio::spawn(async move {
        info!("Starting job to fetch transactions from Powens.");
        
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
}

pub async fn transactions_to_csv_handler(State(app_state): State<AppState>) -> String {
    let account_db = &app_state.account_db;
    let transactions_csv: Vec<TransactionCsv> = (&app_state.transaction_db)
        .data()
        .iter()
        // skip those coming == true
        .filter(|it| !it.coming)
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

pub async fn list_transactions_handler(State(app_state): State<AppState>) -> String {
    serde_json::to_string_pretty(&app_state.transaction_db.data()).unwrap()
}
