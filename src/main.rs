mod google_sheet_api;
mod weather_api;

use teloxide::{
    dispatching::{
        stop_token::AsyncStopToken,
        update_listeners::{self, StatefulListener},
    },
    prelude2::*,
    types::Update,
    utils::command::BotCommand,
};

use dotenv::dotenv;
use google_sheet_api::{check, get_hub};
use reqwest::{StatusCode, Url};
use std::{convert::Infallible, env, error::Error, net::SocketAddr};
use tokio::sync::mpsc;
use tokio_stream::wrappers::UnboundedReceiverStream;
use warp::Filter;

#[derive(BotCommand, Clone)]
#[command(rename = "lowercase", description = "These commands are supported:")]
enum Command {
    #[command(description = "display help message.")]
    Help,
    #[command(description = "check in")]
    CheckIn,
    #[command(description = "check out")]
    CheckOut,
}

#[allow(deprecated)]
async fn answer(
    cx: teloxide::dispatching::UpdateWithCx<AutoSend<Bot>, Message>,
    command: Command,
) -> Result<(), Box<dyn Error + Send + Sync>> {
    let teloxide_bot_ueser =
        env::var("TELOXIDE_BOT_USER").expect("TELOXIDE_BOT_UESER no found");
    if cx.update.chat_id()
        != teloxide_bot_ueser
            .parse::<i64>()
            .expect("parse bot user failed")
    {
        cx.answer(format!("You are not WesleyCh3n. Good bye and Good luck👋"))
            .await?;
        return Ok(());
    }
    match command {
        Command::Help => cx.answer(Command::descriptions()).await?,
        Command::CheckIn => {
            let hub = get_hub().await;
            let msg = check(&hub, "in".into()).await.unwrap();
            cx.answer(format!("🌱 Check in successfully")).await?;
            cx.answer(msg).await?
        }
        Command::CheckOut => {
            let hub = get_hub().await;
            let msg = check(&hub, "out".into()).await.unwrap();
            cx.answer(format!("🌱 Check out successfully")).await?;
            cx.answer(msg).await?
        }
    };

    Ok(())
}

#[tokio::main]
async fn main() {
    dotenv().ok();
    pretty_env_logger::init();
    log::info!("Starting heroku_ping_pong_bot...");

    let bot = Bot::from_env().auto_send();

    // Local Testing
    /* teloxide::commands_repl(bot.clone(), "work-log", answer).await; */

    teloxide::commands_repl_with_listener(
        bot.clone(),
        "work-log",
        answer,
        webhook(bot).await,
    )
    .await;
}

async fn handle_rejection(
    error: warp::Rejection,
) -> Result<impl warp::Reply, Infallible> {
    log::error!("Cannot process the request due to: {:?}", error);
    Ok(StatusCode::INTERNAL_SERVER_ERROR)
}

pub async fn webhook(
    bot: AutoSend<Bot>,
) -> impl update_listeners::UpdateListener<Infallible> {
    // Heroku auto defines a port value
    let teloxide_token = env::var("TELOXIDE_TOKEN")
        .expect("TELOXIDE_TOKEN env variable missing");
    let port: u16 = env::var("PORT")
        .expect("PORT env variable missing")
        .parse()
        .expect("PORT value to be integer");
    // Heroku host example .: "heroku-ping-pong-bot.herokuapp.com"
    let host = env::var("HOST").expect("have HOST env variable");
    let path = format!("bot{}", teloxide_token);
    let url = Url::parse(&format!("https://{}/{}", host, path)).unwrap();

    bot.set_webhook(url).await.expect("Cannot setup a webhook");

    let (tx, rx) = mpsc::unbounded_channel();

    let server = warp::post()
        .and(warp::path(path))
        .and(warp::body::json())
        .map(move |update: Update| {
            tx.send(Ok(update))
                .expect("Cannot send an incoming update from the webhook");

            StatusCode::OK
        })
        .recover(handle_rejection);

    let (stop_token, stop_flag) = AsyncStopToken::new_pair();

    let addr = format!("0.0.0.0:{}", port).parse::<SocketAddr>().unwrap();
    let server = warp::serve(server);
    let (_addr, fut) = server.bind_with_graceful_shutdown(addr, stop_flag);

    // You might want to use serve.key_path/serve.cert_path methods here to
    // setup a self-signed TLS certificate.

    tokio::spawn(fut);
    let stream = UnboundedReceiverStream::new(rx);

    fn streamf<S, T>(state: &mut (S, T)) -> &mut S {
        &mut state.0
    }

    StatefulListener::new(
        (stream, stop_token),
        streamf,
        |state: &mut (_, AsyncStopToken)| state.1.clone(),
    )
}
