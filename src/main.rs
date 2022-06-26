use poise::serenity::async_trait;
use poise::serenity::model::channel::Reaction;
use poise::serenity::model::gateway::{GatewayIntents, Ready};
use poise::serenity::prelude::*;
use poise::FrameworkBuilder;

type Error = Box<dyn std::error::Error + Send + Sync>;
// User data, which is stored and accessible in all command invocations
struct Data {}

struct Handler;
#[async_trait]
impl EventHandler for Handler {
    async fn ready(&self, _: Context, ready: Ready) {
        println!("{} is connected!", ready.user.name);
    }

    async fn reaction_add(&self, ctx: Context, reaction: Reaction) {
        if let Some(msg) = ctx.cache.message(reaction.channel_id, reaction.message_id) {
            println!("{}\n-> {}", msg.content, reaction.emoji);
        }
    }
}

#[tokio::main]
async fn main() {
    let intents = GatewayIntents::GUILD_MESSAGE_REACTIONS
        | GatewayIntents::MESSAGE_CONTENT
        | GatewayIntents::GUILD_MESSAGES
        | GatewayIntents::DIRECT_MESSAGES
        | GatewayIntents::DIRECT_MESSAGE_REACTIONS;

    let framework: FrameworkBuilder<Data, Error> = poise::Framework::build()
        .options(poise::FrameworkOptions::default())
        .token(std::env::var("ROTATRON_TOKEN").expect("missing ROTATRON_TOKEN"))
        .intents(intents)
        .client_settings(|f| f.event_handler(Handler))
        .user_data_setup(move |_ctx, _ready, _framework| Box::pin(async move { Ok(Data {}) }));

    framework.run().await.unwrap();
}
