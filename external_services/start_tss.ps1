Write-Output 'Starting Voice-2-txt-UrukHan TSS service (russian)'
Start-Process -NoNewWindow python -ArgumentList './Voice-2-txt-UrukHan/main.py -m ./models'
