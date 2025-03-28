use crate::genai::gemini_response::GeminiResponse;
use reqwest::Client;
use serde_json::json;
use tracing::{debug, warn};

pub async fn call_gemini(prompt: String) -> Result<String, Box<dyn std::error::Error>> {
    debug!("Prompt: \n{}", prompt);

    // The request URL with the API key
    let api_key = dotenv::var("GEMINI_API_KEY")?;
    let url = format!(
        "https://generativelanguage.googleapis.com/v1beta/models/gemma-3-27b-it:generateContent?key={}",
        api_key
    );

    // JSON body data
    let body = json!({
        "contents": [
            {
                "role": "user",
                "parts": [
                    {
                        "text": prompt
                    }
                ]
            }
        ],
        "generationConfig": {
            "temperature": 0.5,
            "topK": 64,
            "topP": 0.98,
            "maxOutputTokens": 1024,
            "responseMimeType": "text/plain"
        }
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
        let response: GeminiResponse = serde_json::from_str(&response.text().await?)?;
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
