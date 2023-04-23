# AI-Waifu-Vtuber on rust
[Reference project](https://github.com/ardha27/AI-Waifu-Vtuber)

## References
- https://hub.docker.com/r/missuo/deeplx -> integrated

## Required services
- https://github.com/ololoshka2871/Selerio-TTS-server
- https://github.com/ololoshka2871/Voice-2-txt-UrukHan

### Constants
* Discord Voice output 2ch @ 48000 Hz

## Convert PyTorch -> TorchScript for SilerioTTS models
1. Downolad prefered model from `https://models.silero.ai/models/tts/` for example `ru/ru_v3.pt` to `models/ru_ru_v3.pt`
1. Activate venv: `source venv/bin/activate` 
1. Convert model to `TorchScript`: `./pymodules/convert_pt_to_torchscript_model.py ./models/ru_ru_v3.pt` new file `models/ru_ru_v3.pt.torchscript.pt` will be created
1. close venv

## How to use (TODO)
~~1. Create python venv: `python3 -m venv venv`~~
~~1. Activate venv: `source venv/bin/activate`~~
~~1. Install requirements: `pip install -r requirements.txt`~~
~~1. Run Selerio-TTS-server~~
~~1. Run Voice-2-txt-UrukHan~~
~~1. Copy `config.json.init` to `config.json` and fill it~~
~~1. Run `cargo run --release --bin ai-waifu-interactive` or `cargo run --release --bin ai-waifu-discord-bot`~~