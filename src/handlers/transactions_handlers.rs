use crate::app_state::AppState;
use crate::csv::{TransactionCsv, VecToCsv};
use crate::genai::run_ai_guess_on_all_transactions;
use crate::powens::POWENS_DATETIME_FORMAT;
use axum::http::Response;
use axum::{
    body::Body,
    extract::{Query, State},
    http::header,
};
use chrono::{DateTime, NaiveDateTime, Utc};
use serde::Deserialize;
use tracing::error;
use tracing::info;

const PARAM_DATETIME_FORMAT: &str = "%Y-%m-%d_%H-%M-%S";

#[derive(Deserialize)]
pub struct TransactionsToCsvParams {
    last_update: Option<String>,
}

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

pub async fn transactions_to_csv_handler(
    Query(params): Query<TransactionsToCsvParams>,
    State(app_state): State<AppState>,
) -> Response<Body> {
    // parse param
    let mut last_update: Option<DateTime<Utc>> = None;
    if let Some(last_update_str) = &params.last_update {
        if let Ok(last_update_native) =
            NaiveDateTime::parse_from_str(last_update_str, PARAM_DATETIME_FORMAT)
        {
            let last_update_utc: DateTime<Utc> = last_update_native.and_utc();
            last_update = Some(last_update_utc)
        }
    }

    // filter transactions
    let mut transaction = (&app_state.transaction_db).data();
    // keep those coming == false
    transaction.retain(|it| !it.coming);
    // keep if transaction.last_update > param.last_update
    if let Some(last_update_param) = last_update {
        transaction.retain(|it| {
            let last_update_native =
                NaiveDateTime::parse_from_str(&it.last_update, POWENS_DATETIME_FORMAT).unwrap();
            let last_update_utc: DateTime<Utc> = last_update_native.and_utc();

            last_update_utc > last_update_param
        });
    }

    // if empty, end
    if transaction.is_empty() {
        let res_str = if let Some(last_update_str) = &params.last_update {
            format!("No transactions found with last_update > {last_update_str} found")
        } else {
            "No transactions found.".to_string()
        };

        Response::builder()
            .status(200)
            .body(Body::from(res_str))
            .unwrap()
    }
    // export csv
    else {
        // find the biggest last_update in transactions, use it to create a download file name
        let mut biggest_last_update: Option<DateTime<Utc>> = None;
        for transaction in transaction.iter() {
            let last_update_native =
                NaiveDateTime::parse_from_str(&transaction.last_update, POWENS_DATETIME_FORMAT)
                    .unwrap();
            let last_update_utc: DateTime<Utc> = last_update_native.and_utc();
            if let Some(biggest_last_update_inner) = biggest_last_update {
                if last_update_utc > biggest_last_update_inner {
                    biggest_last_update = Some(last_update_utc);
                }
            } else {
                biggest_last_update = Some(last_update_utc);
            }
        }

        // convert to Transaction to TransactionCsv
        let account_db = &app_state.account_db;
        let transactions_csv: Vec<TransactionCsv> = transaction
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

        // result csv
        let result = transactions_csv.to_csv();

        // convert the result into a http body
        let body = Body::from(result);

        let filename = format!(
            "transactions {}.csv",
            biggest_last_update.unwrap().format(PARAM_DATETIME_FORMAT)
        );

        Response::builder()
            .status(200) // Set status code as needed
            .header(header::CONTENT_TYPE, "text/csv")
            .header(
                header::CONTENT_DISPOSITION,
                format!("attachment; filename=\"{}\"", filename),
            )
            .body(body)
            .unwrap()
    }
}

pub async fn list_transactions_handler(State(app_state): State<AppState>) -> String {
    serde_json::to_string_pretty(&app_state.transaction_db.data()).unwrap()
}
