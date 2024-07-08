# Rustonlinellm

Currently the Rust online LLM is working and able to search online if built locally in docker and with the following steps.

1. In your local terminal run
2. `docker build -t openai-api-rust:latest .`
3. `docker run -p 8080:8080 openai-api-rust:latest`
4. In a new terminal run the following curl command.
   `curl -X POST http://localhost:8080/openai/deployments/pt_rekoncile_onlinellm/chat/completions \
-H "Content-Type: application/json" \
-H "api-key: 99d8e7d8c355486583e8ab633d7ff06b" \
-d '{
  "model": "pt_rekoncile_onlinellm",
  "messages": [{"role": "user", "content": "What is the latest new in generative AI?"}]

}'`

Note: the API key will be deprecated within 2-3 working days of sending the take-home task for review. 
