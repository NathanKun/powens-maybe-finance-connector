use axum::extract::State;
use crate::app_state::AppState;
use crate::csv::{AccountCsv, VecToCsv};

pub async fn list_accounts_handler(State(app_state): State<AppState>) -> String {
    serde_json::to_string_pretty(&app_state.account_db.data()).unwrap()
}

pub async fn accounts_to_csv_handler(State(app_state): State<AppState>) -> String {
    let accounts_csv: Vec<AccountCsv> = (&app_state.account_db)
        .data()
        .iter()
        .map(|it| it.into())
        .collect();
    accounts_csv.to_csv()
}
