use actix_web::{web, App, HttpServer, Responder, HttpResponse};
use serde::{Deserialize, Serialize};
use reqwest::Client;
use scraper::{Html, Selector};
use dotenv::dotenv;
use std::env;
use actix_web::http::header::HeaderValue;

#[derive(Deserialize)]
struct ChatCompletionRequest {
    model: String,
    messages: Vec<Message>,
}

#[derive(Deserialize, Serialize, Clone)]
struct Message {
    role: String,
    content: String,
}

#[derive(Serialize)]
struct ChatCompletionResponse {
    id: String,
    object: String,
    created: u64,
    model: String,
    choices: Vec<Choice>,
}

#[derive(Serialize)]
struct Choice {
    index: u32,
    message: Message,
    finish_reason: String,
}

async fn web_search(query: &str) -> Result<Vec<String>, Box<dyn std::error::Error>> {
    let client = Client::new();
    let url = format!("https://html.duckduckgo.com/html/?q={}", query);
    let body = client.get(&url).send().await?.text().await?;

    let document = Html::parse_document(&body);
    let selector = Selector::parse(".result__snippet").unwrap();

    let results: Vec<String> = document
        .select(&selector)
        .take(5)
        .map(|element| element.text().collect::<String>())
        .collect();

    Ok(results)
}

fn verify_api_key(api_key: &HeaderValue) -> bool {
    let valid_key = std::env::var("AZURE_OPENAI_KEY").expect("AZURE_OPENAI_KEY not set");
    api_key.to_str().unwrap_or("") == valid_key
}

async fn chat_completions(req: web::Json<ChatCompletionRequest>, api_key: web::Header<String>) -> impl Responder {
    if !verify_api_key(api_key.as_ref()) {
        return HttpResponse::Unauthorized().json(serde_json::json!({
            "error": {
                "message": "Invalid API key",
                "type": "invalid_request_error",
                "param": null,
                "code": null
            }
        }));
    }

    let user_message = req.messages.last().unwrap().content.clone();
    let mut messages = req.messages.clone();

    if req.model.contains("_onlinellm") {
        let search_results = web_search(&user_message).await.unwrap_or_default();
        let relevant_info = search_results.join("\n");
        
        messages.insert(0, Message {
            role: "system".to_string(),
            content: format!("Relevant information from web search:\n{}", relevant_info),
        });
    }

    if !messages.iter().any(|m| m.role == "system") {
        messages.insert(0, Message {
            role: "system".to_string(),
            content: "You are a knowledgeable AI assistant. Provide detailed, informative answers with examples and context when appropriate. Aim for responses that are at least 3-4 sentences long.".to_string(),
        });
    }

    let client = Client::new();
    let azure_endpoint = std::env::var("AZURE_OPENAI_ENDPOINT").expect("AZURE_OPENAI_ENDPOINT not set");
    let azure_key = std::env::var("AZURE_OPENAI_KEY").expect("AZURE_OPENAI_KEY not set");
    let deployment_name = std::env::var("AZURE_OPENAI_DEPLOYMENT_NAME").expect("AZURE_OPENAI_DEPLOYMENT_NAME not set");
    let api_version = std::env::var("AZURE_OPENAI_API_VERSION").expect("AZURE_OPENAI_API_VERSION not set");

    let url = format!("{}/openai/deployments/{}/chat/completions?api-version={}", azure_endpoint, deployment_name, api_version);

    let azure_request = serde_json::json!({
        "messages": messages,
        "max_tokens": 1200,
        "temperature": 0.7,
        "frequency_penalty": 0,
        "presence_penalty": 0,
        "top_p": 0.95,
        "stop": null
    });

    let azure_response = client.post(&url)
        .header("api-key", azure_key)
        .json(&azure_request)
        .send()
        .await
        .expect("Failed to send request to Azure OpenAI")
        .json::<serde_json::Value>()
        .await
        .expect("Failed to parse Azure OpenAI response");

    let response_content = azure_response["choices"][0]["message"]["content"].as_str().unwrap_or("").to_string();

    let response = ChatCompletionResponse {
        id: azure_response["id"].as_str().unwrap_or("").to_string(),
        object: "chat.completion".to_string(),
        created: azure_response["created"].as_u64().unwrap_or(0),
        model: req.model.clone(),
        choices: vec![Choice {
            index: 0,
            message: Message {
                role: "assistant".to_string(),
                content: response_content,
            },
            finish_reason: azure_response["choices"][0]["finish_reason"].as_str().unwrap_or("").to_string(),
        }],
    };

    HttpResponse::Ok().json(response)
}

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    dotenv().ok();
    let port = env::var("PORT").unwrap_or_else(|_| "8080".to_string());
    let address = format!("0.0.0.0:{}", port);
    println!("Server running at http://{}", address);
    HttpServer::new(|| {
        App::new()
            .route("/openai/deployments/{model_name}/chat/completions",
                web::post().to(chat_completions))
    })
    .bind(address)?
    .run()
    .await
}