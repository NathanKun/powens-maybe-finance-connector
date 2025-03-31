use crate::genai::gemini_response::GeminiResponse;
use reqwest::Client;
use serde_json::json;
use tracing::{trace, warn};

const MODEL_ID: &str = "gemini-2.0-flash";

pub async fn call_gemini(
    system_prompt: String,
    user_prompt: String,
) -> Result<String, Box<dyn std::error::Error>> {
    trace!("system_prompt: \n{}", system_prompt);
    trace!("user_prompt: \n{}", user_prompt);

    // The request URL with the API key
    let api_key = dotenv::var("GEMINI_API_KEY")?;
    let url = format!(
        "https://generativelanguage.googleapis.com/v1beta/models/{MODEL_ID}:generateContent?key={api_key}"
    );

    // JSON body data
    let body = json!({
        "contents": [
          {
            "role": "user",
            "parts": [
              {
                "text": user_prompt
              },
            ]
          },
        ],
        "systemInstruction": {
          "parts": [
            {
                "text": system_prompt
            },
          ]
        },
        "generationConfig": {
          "temperature": 0.5,
          "topP": 1,
          "responseMimeType": "application/json",
        },
    });

    // Create an HTTP client
    let client = Client::new();

    // Perform the POST request
    let response = client
        .post(&url)
        .header("Content-Type", "application/json")
        .json(&body)
        .send()
        .await?;

    // Check for success and print the response
    if response.status().is_success() {
        let text = &response.text().await?;
        trace!("response: \n{}", &text);
        let response: GeminiResponse = serde_json::from_str(text)?;
        Ok(response
            .candidates
            .get(0)
            .unwrap()
            .content
            .parts
            .get(0)
            .unwrap()
            .text
            .clone())
    } else {
        warn!(
            "Gemini call failed with status: {} {}",
            response.status(),
            response.text().await?
        );
        Err("Gemini call failed".into())
    }
}
