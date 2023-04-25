Write-Output 'Starting py_services/Selerio-TTS-server'
Start-Process -NoNewWindow python -ArgumentList './py_services/Selerio-TTS-server/main.py -m models ru ru_v3'

Write-Output 'Starting py_services/Voice-2-txt-UrukHan'
Start-Process -NoNewWindow python -ArgumentList './py_services/Voice-2-txt-UrukHan/main.py -m models'

Write-Output 'Starting llama_cpp.server'
$env:MODEL=".\\models\\7B\\ggml-model-f16.bin"
Start-Process -NoNewWindow python -ArgumentList '-m llama_cpp.server'