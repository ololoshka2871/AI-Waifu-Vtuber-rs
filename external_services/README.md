# External services for AI-Waifu-Vtuber
Yu can run this servises on the same machine as AI-Waifu-Vtuber or on another one in the same network.


## Silerio-TTS-server
TTS service based on [Selerio-TTS](https://models.silero.ai/models/tts) models.


## Voice-2-txt-UrukHan STT
Voice recognition service based on [UrukHan/wav2vec2-russian](https://huggingface.co/UrukHan/wav2vec2-russian) and [UrukHan/t5-russian-spell](https://huggingface.co/UrukHan/t5-russian-spell) models fo russian language.


## llama-cpp-python
[Python bindings for llama.cpp](https://github.com/abetlen/llama-cpp-python)

### Usage
1. Download LLaMa model by magnet link with hash: <cdee3052d85c697b84f4c1192f43a2276c0daea0>
2. Convert it to ggml format see [this](https://github.com/rustformers/llama-rs#getting-the-weights) and place to `models` directory
    Example: `./models/7B/ggml-model-f16.bin`


## How to use
1. Create python venv: `python3 -m venv venv`
2. Activate venv: for Linux `source venv/bin/activate` or for Windows `.\venv\Scripts\activate`
3. Install requirements: `pip install -r requirements.txt`
4. Check and run scrits to start one ore more services:
    - `start_silerio.sh`
    - `start_tss-openai-whisper.sh`
    - `start_llama.sh`