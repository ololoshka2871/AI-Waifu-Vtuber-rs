use tokio::io::{self, AsyncBufReadExt, AsyncWriteExt, BufReader};

use ai_waifu::{
    dispatcher::{AIRequest, Dispatcher},
    dummy_ai::DummyAI,
    google_translator::GoogleTranslator,
    handler::Handler,
    request::Request,
    config::Config,
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

    let ai = Box::new(DummyAI);
    let en_ai = GoogleTranslator::new(ai, Some("ru".to_string()), None).await;

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
