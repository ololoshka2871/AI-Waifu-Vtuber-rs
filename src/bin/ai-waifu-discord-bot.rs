use std::env;

use serenity::{client::Client, framework::StandardFramework, prelude::GatewayIntents};

use songbird::{driver::DecodeMode, Config, SerenityInit};

use tracing::{info, warn};

use ai_waifu::{
    dummy_ai::DummyAI, google_translator::GoogleTranslator, handler::Handler, request::Request,
    dispatcher::Dispatcher,
};

#[tokio::main]
async fn main() {
    tracing_subscriber::fmt::init();

    // Configure the client with your Discord bot token in the environment.
    let token = env::var("DISCORD_TOKEN").expect("Expected a token in the environment");
    let channel_name_part =
        env::var("CHANNEL_NAME_PART").expect("Expected a channel name part in the environment");

    let framework = StandardFramework::new()
        //.configure(|c| c
        //    .prefix("~"))
        //.group(&GENERAL_GROUP); --> struct General
        ;

    let intents = GatewayIntents::non_privileged()
        | GatewayIntents::MESSAGE_CONTENT
        | GatewayIntents::GUILD_VOICE_STATES;

    // Here, we need to configure Songbird to decode all incoming voice packets.
    // If you want, you can do this on a per-call basis---here, we need it to
    // read the audio data that other people are sending us!
    let songbird_config = Config::default().decode_mode(DecodeMode::Decode);

    let (text_request_channel_tx, mut text_request_channel_rx) =
        tokio::sync::mpsc::channel::<Request>(1);

    let ai = Box::new(DummyAI);
    let en_ai = GoogleTranslator::new(ai, None, None).await;

    let mut dispatcher = Dispatcher::new(Box::new(en_ai));    

    tokio::spawn(async move {
        while let Some(req) = text_request_channel_rx.recv().await {
            warn!("Received request: {:?}", req);
        }
    });

    //tokio::spawn(async move {
    //    let req = TestRequest {
    //        request: "Мама мыла раму.".to_string(),
    //        author: "Master".to_string(),
    //    };
    //
    //    match dispatcher.try_process_request(Box::new(req)).await {
    //        Ok(resp) => {
    //            warn!("Response: {}", resp);
    //        }
    //        Err(err) => {
    //            warn!("Error: {:?}", err);
    //        }
    //    }
    //});

    let mut bot = Client::builder(&token, intents)
        .event_handler(Handler::new(text_request_channel_tx, channel_name_part))
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
