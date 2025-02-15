use actix_web::{web, App, HttpServer, Responder, HttpResponse, dev::Payload, Error as ActixError, FromRequest, HttpRequest};
use serde::{Deserialize, Serialize};
use reqwest::Client;
use scraper::{Html, Selector};
use dotenv::dotenv;
use std::env;
use futures::future::{ready, Ready};
use log::{info, error}; // Import logging macros
use env_logger; // Import the env_logger crate for initialization


#[derive(Deserialize, Debug)]  // Added Debug here
struct ChatCompletionRequest {
    model: String,
    messages: Vec<Message>,
}

#[derive(Deserialize, Serialize, Clone, Debug)]  // Added Debug here
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

struct ApiKey(String);

impl FromRequest for ApiKey {
    type Error = ActixError;
    type Future = Ready<Result<Self, Self::Error>>;

    fn from_request(req: &HttpRequest, _: &mut Payload) -> Self::Future {
        let api_key = req
            .headers()
            .get("api-key")
            .and_then(|h| h.to_str().ok())
            .map(String::from);

        match api_key {
            Some(key) => ready(Ok(ApiKey(key))),
            None => ready(Err(actix_web::error::ErrorUnauthorized("Missing API key"))),
        }
    }
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

fn verify_api_key(api_key: &str) -> bool {
    let valid_key = std::env::var("AZURE_OPENAI_KEY").expect("AZURE_OPENAI_KEY not set");
    api_key == valid_key
}

async fn chat_completions(req: web::Json<ChatCompletionRequest>, api_key: ApiKey) -> impl Responder {
    log::info!("Received request: {:?}", req);
    if !verify_api_key(&api_key.0) {
        log::error!("Invalid API key: {}", api_key.0);
        return HttpResponse::Unauthorized().json(serde_json::json!({
            "error": "Invalid API key"
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
    let azure_endpoint = env::var("AZURE_OPENAI_ENDPOINT").expect("AZURE_OPENAI_ENDPOINT not set");
    let azure_key = env::var("AZURE_OPENAI_KEY").expect("AZURE_OPENAI_KEY not set");
    let deployment_name = env::var("AZURE_OPENAI_DEPLOYMENT_NAME").expect("AZURE_OPENAI_DEPLOYMENT_NAME not set");
    let api_version = env::var("AZURE_OPENAI_API_VERSION").expect("AZURE_OPENAI_API_VERSION not set");

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
    env_logger::init();

    info!("Starting application...");
    info!("Current working directory: {:?}", std::env::current_dir());

    dotenv().ok();

    let port = env::var("PORT").unwrap_or_else(|_| {
        info!("PORT not set, defaulting to 8080");
        "8080".to_string()
    });

    info!("Attempting to bind to address: 0.0.0.0:{}", port);

    let azure_key = env::var("AZURE_OPENAI_KEY").unwrap_or_else(|e| {
        error!("AZURE_OPENAI_KEY not set: {}", e);
        panic!("AZURE_OPENAI_KEY not set");
    });

    let azure_endpoint = env::var("AZURE_OPENAI_ENDPOINT").unwrap_or_else(|e| {
        error!("AZURE_OPENAI_ENDPOINT not set: {}", e);
        panic!("AZURE_OPENAI_ENDPOINT not set");
    });

    info!("Starting Actix Web Server...");

    match HttpServer::new(|| {
        App::new()
            .route("/openai/deployments/{model_name}/chat/completions", web::post().to(chat_completions))
    })
    .bind(("0.0.0.0", port.parse().unwrap())) {
        Ok(server) => {
            info!("Successfully bound to address: 0.0.0.0:{}", port);
            server.run().await
        },
        Err(e) => {
            error!("Failed to bind to address: 0.0.0.0:{}. Error: {}", port, e);
            Err(e)
        }
    }
}