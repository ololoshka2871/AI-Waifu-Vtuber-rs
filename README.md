# AI-Waifu-Vtuber on rust
[Reference project](https://github.com/ardha27/AI-Waifu-Vtuber)

## References
- https://hub.docker.com/r/missuo/deeplx -> Translation using [DeepL](https://www.deepl.com/translator)

## External services
See `external_services` directory dor details

### Constants
* Discord Voice output 2ch @ 48000 Hz

## How to use
### Config
1. Copy `config.example.json` to `config.json`
2. Fill `config.json` with your data

### Run
1. Start selected services (see `external_services` directory)
2. Run `cargo run --release --bin ai-waifu-vtuber` or `cargo run --release --bin ai-waifu-interactive`