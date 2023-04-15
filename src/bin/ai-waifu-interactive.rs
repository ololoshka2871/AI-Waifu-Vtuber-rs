use tokio::io::{self, AsyncBufReadExt, AsyncWriteExt, BufReader};

use ai_waifu::{
    chatgpt::ChatGPT,
    config::Config,
    deeplx_translate::DeepLxTranslator,
    dispatcher::{AIRequest, Dispatcher},
    //google_translator::GoogleTranslator,
};

struct InteractiveRequest {
    request: String,
}

impl AIRequest for InteractiveRequest {
    fn request(&self) -> String {
        self.request.clone()
    }

    fn author(&self) -> String {
        "interactive".to_string()
    }
}

#[tokio::main]
async fn main() {
    tracing_subscriber::fmt::init();

    let config = Config::load();

    let ai = ChatGPT::new(config.openai_token, config.initial_prompt);

    // use this image: https://hub.docker.com/r/missuo/deeplx
    let en_ai = DeepLxTranslator::new(Box::new(ai), Some("ru".to_string()), None, "http://localhost:1188/translate".to_string()).await;

    let mut dispatcher = Dispatcher::new(Box::new(en_ai));

    let stdin = io::stdin();
    let mut stdout = io::stdout();
    let mut reader = BufReader::new(stdin);
    let mut buffer = String::new();

    loop {
        // prompt
        stdout.write("> ".as_bytes()).await.unwrap();
        stdout.flush().await.unwrap();

        // read a line
        let _size = reader.read_line(&mut buffer).await.unwrap();

        let res = dispatcher
            .try_process_request(Box::new(InteractiveRequest {
                request: buffer.trim().to_string(),
            }))
            .await
            .unwrap();

        // write the line
        stdout
            .write(format!("< {}\n", res).as_bytes())
            .await
            .unwrap();

        // clear buffer
        buffer.clear();
    }
}
