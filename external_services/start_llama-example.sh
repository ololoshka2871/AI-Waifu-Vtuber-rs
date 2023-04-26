. ./venv/Scripts/activate
echo 'Starting llama_cpp.server'
MODEL="./models/7B/ggml-model-f16.bin" python -m llama_cpp.server