# Rustonlinellm

Currently the Rust online LLM is working and able to search online if built locally in docker and with the following steps.

# OpenAI API Rust Implementation

This project implements a Rust-based API that interacts with Azure OpenAI services and includes web search functionality.

## Features

- Chat completion API endpoint
- Web search integration
- Azure OpenAI service integration
- API key verification

## Prerequisites

- Rust
- Actix-web
- Reqwest
- Scraper
- Dotenv

## Environment Variables

The following environment variables need to be set:

- `AZURE_OPENAI_ENDPOINT`
- `AZURE_OPENAI_KEY`
- `AZURE_OPENAI_DEPLOYMENT_NAME`
- `AZURE_OPENAI_API_VERSION`
- `PORT` (optional, defaults to 8081)

## API Endpoint

POST `/openai/deployments/{model_name}/chat/completions`

### Request Body

```json
{
  "model": "string",
  "messages": [
    {
      "role": "string",
      "content": "string"
    }
  ]
}

## STEPS

1. In your local terminal run `docker pull pulkittalwar/openai-api-rust:latest`
2. `docker run -p 8081:8080 pulkittalwar/openai-api-rust:latest`
4. In a new terminal run the following curl command.
curl -X POST http://localhost:8081/openai/deployments/pt_rekoncile_onlinellm/chat/completions \
-H "Content-Type: application/json" \
-H "api-key: <PLEASE_SEE_EMAIL_PYTHON_CODE> \
-d '{
  "model": "pt_rekoncile_onlinellm",
  "messages": [{"role": "user", "content": "Hello, how are you?"}]
}'


Note: the API key will be deprecated within 2-3 working days of sending the take-home task for review. 
