mod api;
mod gemini_response;

use crate::genai::api::call_gemini;
use crate::powens::{Transaction, TransactionType};
use regex::Regex;
use serde::{Deserialize, Serialize};
use std::fs;
use tracing::{info};
use tracing::log::debug;
use crate::app_state::AppState;
use crate::db::TransactionExtras;

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct SimplifiedTransaction {
    pub value: f64,
    pub original_wording: String,
    pub stemmed_wording: String,
    pub transaction_type: TransactionType,
}

impl From<&Transaction> for SimplifiedTransaction {
    fn from(transaction: &Transaction) -> Self {
        Self {
            value: transaction.value,
            original_wording: transaction.original_wording.clone(),
            stemmed_wording: transaction.stemmed_wording.clone(),
            transaction_type: transaction.transaction_type.clone(),
        }
    }
}

const PROMPT: &str = r#"
You are an expert transaction classifier designed to categorize financial transactions into predefined categories and subcategories.

**Input:**

**Category and Subcategory Definitions (JSON):**
Income
```json
{INCOME_JSON}
```


Expenses
```json
{EXPENSES_JSON}
```


**Instructions:**

1.  Analyze the provided transaction description.
2.  Match the transaction to the most appropriate category and, if applicable, subcategory from the provided JSON.
3.  The "examples" field in the JSON is a list of examples of transactions wording that fit into the category, but it is not exhaustive, try your best to guess a category, if really unsure, use "Other Expenses".
4.  For positive transactions, use the "Income" json. For negative transactions, use the "Expenses" json.
5.  If a match is found, return a JSON array containing the category and, if relevant, the subcategory. Do not include "Expenses" or "Income", they are not a category.
7.  Assume the transaction description may be in French.

**Output (JSON Array)**

"#;

pub async fn ai_guess_transaction_categories(
    transaction: &Transaction,
) -> Result<Vec<String>, Box<dyn std::error::Error>> {
    let mut input_transaction: SimplifiedTransaction = transaction.into();

    // replace sensitive data
    let regex_replaces: [(Regex, &str); 3] = [
        (Regex::new(r"CARTE X\d{4}").unwrap(), "CARTE X0000"), // card number
        (Regex::new(r"\d{5,99}").unwrap(), "00000"),           // numbers
        (Regex::new(r"\d{2}/\d{2}").unwrap(), "01/01"),        // date
    ];

    for (regex, replace) in regex_replaces.iter() {
        input_transaction.original_wording = regex
            .replace_all(&input_transaction.original_wording, *replace)
            .to_string();
        input_transaction.stemmed_wording = regex
            .replace_all(&input_transaction.stemmed_wording, *replace)
            .to_string();
    }

    // load income.json & expenses.json
    let income_json = fs::read_to_string("ai-prompts/income.json")
        .unwrap_or_else(|_| fs::read_to_string("ai-prompts/income.json.example").unwrap());

    let expenses_json = fs::read_to_string("ai-prompts/expenses.json")
        .unwrap_or_else(|_| fs::read_to_string("ai-prompts/expenses.json.example").unwrap());

    // final prompt
    let system_prompt = PROMPT
        .to_string()
        .replace("{INCOME_JSON}", &income_json)
        .replace("{EXPENSES_JSON}", &expenses_json);
    let user_prompt = serde_json::to_string(&input_transaction)?;

    // call gemini
    info!(
        "Calling Gemini to guess category of transaction {}",
        transaction.id
    );
    debug!("{:#?}", input_transaction);
    let text = call_gemini(system_prompt, user_prompt).await?;

    // parse response text
    if text.starts_with("[") && text.ends_with("]") {
        let json: serde_json::Value = serde_json::from_str(&text)?;
        let mut categories: Vec<String> = json
            .as_array()
            .unwrap()
            .iter()
            .map(|it| String::from(it.as_str().unwrap()))
            .collect();

        // if the first string is "Expenses" or "Income", remove it
        if let Some(category) = categories.get(0) {
            if category == "Expenses" || category == "Income" {
                categories.remove(0);
            }
        }

        debug!("Gemini return category: {:?}", categories);
        return Ok(categories);
    }

    Err("Failed to parse JSON".into())
}

pub async fn run_ai_guess_on_all_transactions(
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

        // avoid rate limit if free tier
        // tokio::time::sleep(std::time::Duration::from_secs(4)).await;
    }

    info!("AI guessing finished.");
    Ok(())
}
