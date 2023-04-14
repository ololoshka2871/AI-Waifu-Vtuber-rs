use tokio::io::{self, BufReader, AsyncBufReadExt, AsyncWriteExt};

use ai_waifu::{
    dispatcher::Dispatcher, dummy_ai::DummyAI, google_translator::GoogleTranslator,
    handler::Handler, request::Request,
};

#[tokio::main]
async fn main() {
    let ai = Box::new(DummyAI);
    let en_ai = GoogleTranslator::new(ai, None, None).await;

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

        let trimmed = buffer.trim();

        // write the line
        stdout.write(format!("< {}\n", trimmed).as_bytes()).await.unwrap();

        // clear buffer
        buffer.clear();
    }
}
