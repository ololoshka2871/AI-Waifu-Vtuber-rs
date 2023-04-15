mod discord_ai_request;
mod discord_event_handler;
mod text_control;
mod voice_receiver;

use serenity::{client::Client, framework::StandardFramework, prelude::GatewayIntents};

use songbird::{driver::DecodeMode, Config, SerenityInit};

use tracing::{debug, error, info, warn};

use ai_waifu::{
    chatgpt::ChatGPT, config::Config as BotConfig, dispatcher::Dispatcher,
    google_translator::GoogleTranslator,
};
use discord_event_handler::DiscordEventHandler;
use text_control::{TextRequest, TextResponse};

#[tokio::main]
async fn main() {
    tracing_subscriber::fmt::init();

    let config = BotConfig::load();

    let intents = GatewayIntents::non_privileged()
        | GatewayIntents::MESSAGE_CONTENT
        | GatewayIntents::GUILD_VOICE_STATES;

    let (control_request_channel_tx, mut control_request_channel_rx) =
        tokio::sync::mpsc::channel::<TextRequest>(1);

    let (text_responce_channel_tx, text_responce_channel_rx) =
        tokio::sync::mpsc::channel::<TextResponse>(1);

    let ai = ChatGPT::new(config.openai_token, config.initial_prompt);
    let en_ai = GoogleTranslator::new(Box::new(ai), Some("ru".to_string()), None).await;

    // test google translate
    let _ = en_ai.translate("緑色", None, "ru").await.map_err(|e| panic!("Google translate error: {}", e));
    info!("Translating works");

    let mut dispatcher = Dispatcher::new(Box::new(en_ai));

    tokio::spawn(async move {
        while let Some(req) = control_request_channel_rx.recv().await {
            match req {
                TextRequest::TextRequest(msg_id, channel_id, user, req) => {
                    let request = discord_ai_request::DiscordAIRequest { request: req, user };
                    info!("{}", request);
                    match dispatcher.try_process_request(Box::new(request)).await {
                        Ok(resp) => {
                            let text_resp = TextResponse {
                                req_msg_id: Some(msg_id),
                                channel_id: channel_id,
                                text: resp.clone(),
                            };
                            match text_responce_channel_tx.send(text_resp).await {
                                Ok(_) => {
                                    debug!("Response: {}", resp)
                                }
                                Err(err) => {
                                    error!("Error send discord responce: {:?}", err);
                                }
                            }
                        }
                        Err(err) => {
                            error!("AI Error: {:?}", err);
                        }
                    }
                }
                r => {
                    warn!("Not implemented yet: {:?}", r);
                }
            }
        }
    });

    let framework = StandardFramework::new();

    // Here, we need to configure Songbird to decode all incoming voice packets.
    // If you want, you can do this on a per-call basis---here, we need it to
    // read the audio data that other people are sending us!
    let songbird_config = Config::default().decode_mode(DecodeMode::Decode);

    let mut bot = Client::builder(&config.discord_token, intents)
        .event_handler(DiscordEventHandler::new(
            control_request_channel_tx,
            text_responce_channel_rx,
            config.channel_whitelist,
        ))
        .framework(framework)
        .register_songbird_from_config(songbird_config)
        .await
        .expect("Err creating client");

    let _ = bot
        .start()
        .await
        .map_err(|why| info!("Client ended: {:?}", why));
}

//#[group]
//#[commands(join, leave, ping)]
//struct General;
//
//#[command]
//#[only_in(guilds)]
//async fn join(ctx: &Context, msg: &Message, mut args: Args) -> CommandResult {
//    let connect_to = match args.single::<u64>() {
//        Ok(id) => ChannelId(id),
//        Err(_) => {
//            check_msg(msg.reply(ctx, "Requires a valid voice channel ID be given").await);
//
//            return Ok(());
//        },
//    };
//
//    let guild = msg.guild(&ctx.cache).unwrap();
//    let guild_id = guild.id;
//
//    let manager = songbird::get(ctx).await
//        .expect("Songbird Voice client placed in at initialisation.").clone();
//
//    let (handler_lock, conn_result) = manager.join(guild_id, connect_to).await;
//
//    if let Ok(_) = conn_result {
//        // NOTE: this skips listening for the actual connection result.
//        let mut handler = handler_lock.lock().await;
//
//        handler.add_global_event(
//            CoreEvent::SpeakingStateUpdate.into(),
//            Receiver::new(),
//        );
//
//        handler.add_global_event(
//            CoreEvent::SpeakingUpdate.into(),
//            Receiver::new(),
//        );
//
//        handler.add_global_event(
//            CoreEvent::VoicePacket.into(),
//            Receiver::new(),
//        );
//
//        handler.add_global_event(
//            CoreEvent::RtcpPacket.into(),
//            Receiver::new(),
//        );
//
//        handler.add_global_event(
//            CoreEvent::ClientDisconnect.into(),
//            Receiver::new(),
//        );
//
//        check_msg(msg.channel_id.say(&ctx.http, &format!("Joined {}", connect_to.mention())).await);
//    } else {
//        check_msg(msg.channel_id.say(&ctx.http, "Error joining the channel").await);
//    }
//
//    Ok(())
//}
//
//#[command]
//#[only_in(guilds)]
//async fn leave(ctx: &Context, msg: &Message) -> CommandResult {
//    let guild = msg.guild(&ctx.cache).unwrap();
//    let guild_id = guild.id;
//
//    let manager = songbird::get(ctx).await
//        .expect("Songbird Voice client placed in at initialisation.").clone();
//    let has_handler = manager.get(guild_id).is_some();
//
//    if has_handler {
//        if let Err(e) = manager.remove(guild_id).await {
//            check_msg(msg.channel_id.say(&ctx.http, format!("Failed: {:?}", e)).await);
//        }
//
//        check_msg(msg.channel_id.say(&ctx.http,"Left voice channel").await);
//    } else {
//        check_msg(msg.reply(ctx, "Not in a voice channel").await);
//    }
//
//    Ok(())
//}
//
//#[command]
//async fn ping(ctx: &Context, msg: &Message) -> CommandResult {
//    check_msg(msg.channel_id.say(&ctx.http,"Pong!").await);
//
//    Ok(())
//}
//
///// Checks that a message successfully sent; if not, then logs why to stdout.
//fn check_msg(result: SerenityResult<Message>) {
//    if let Err(why) = result {
//        println!("Error sending message: {:?}", why);
//    }
//}
