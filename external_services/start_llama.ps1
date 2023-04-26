Write-Output 'Starting llama_cpp.server'
$env:MODEL=".\\models\\7B\\ggml-model-f16.bin"
Start-Process -NoNewWindow python -ArgumentList '-m llama_cpp.server'