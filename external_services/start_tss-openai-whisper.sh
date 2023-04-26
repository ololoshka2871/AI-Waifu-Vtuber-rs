. ./venv/Scripts/activate
echo 'Starting Voice-2-txt-openai-whisper TSS service'
python ./Voice-2-txt-faster-whisper/main.py -m medium -d ./models
