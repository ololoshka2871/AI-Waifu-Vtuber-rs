. ./venv/Scripts/activate
echo 'Starting Voice-2-txt-openai-whisper TSS service'
# if use cuda yu mas provide path to torch libs dir in windows
export SPECIAL_PATH="$PATH:$PWD/venv/Lib/site-packages/torch/lib"
PATH=$SPECIAL_PATH python ./Voice-2-txt-faster-whisper/main.py -m medium -d ./models -c cpu
