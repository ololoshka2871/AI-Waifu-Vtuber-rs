Write-Output 'Starting Silerio-TTS-server with russian voice model'
Start-Process -NoNewWindow python -ArgumentList './Selerio-TTS-server/main.py -m ./models ru ru_v3'
