use image::write_buffer_with_format;
use image::ImageFormat;
use log::info;
use phf::phf_map;
use poise::serenity::async_trait;
use poise::serenity::model::channel::{Reaction, ReactionType};
use poise::serenity::model::gateway::{GatewayIntents, Ready};
use poise::serenity::prelude::*;
use poise::serenity_prelude::Mention;
use poise::serenity_prelude::{ChannelId, MessageId};
use poise::FrameworkBuilder;
use std::io::{Cursor, Seek, SeekFrom};

type Error = Box<dyn std::error::Error + Send + Sync>;
static ROTATIONS: phf::Map<&'static str, i32> = phf_map! {
    "↪️" => 270,
    "↩️" => 90,
    "⤵️" => 90,
    "⤴️" => 270,
    "🔃" => 180,
    "🔁" => 180,
    "🔄" => 180,
    "🗘" => 180,
};

// User data, which is stored and accessible in all command invocations
struct Data {}

fn get_rotation(reaction: &Reaction) -> Option<i32> {
    if let ReactionType::Unicode(emoji) = &reaction.emoji {
        return ROTATIONS.get(emoji).cloned();
    }
    None
}

async fn get_image_url_and_author(
    ctx: &Context,
    channel_id: ChannelId,
    message_id: MessageId,
) -> Option<(String, Mention)> {
    // returns the URL of the first image encountered in the attachments of the message if any
    if let Ok(msg) = ctx.http.get_message(channel_id.0, message_id.0).await {
        for attachment in msg.attachments {
            // hacky way to check if the attachment is an image
            if attachment.width.is_some() {
                return Some((attachment.url, msg.author.mention()));
            }
        }
    }
    None
}

struct Handler;
#[async_trait]
impl EventHandler for Handler {
    async fn ready(&self, _: Context, ready: Ready) {
        println!("{} is connected!", ready.user.name);
    }

    async fn reaction_add(&self, ctx: Context, reaction: Reaction) {
        if let Some(rotation) = get_rotation(&reaction) {
            if let Some((url, author)) =
                get_image_url_and_author(&ctx, reaction.channel_id, reaction.message_id).await
            {
                reaction
                    .channel_id
                    .broadcast_typing(&ctx.http)
                    .await
                    .unwrap();
                // get name and format from URL
                let img_name = url.clone().split_off(url.rfind("/").unwrap() + 1);
                info!("Rotating {} by {} degrees", img_name, rotation);
                let format = ImageFormat::from_path(&img_name).unwrap();
                // download the image
                let resp = reqwest::get(url).await.unwrap();
                // load the image as a image::DynamicImage
                let img_bytes: Vec<_> = resp.bytes().await.unwrap().into_iter().collect();
                let mut img =
                    image::load_from_memory_with_format(img_bytes.as_slice(), format).unwrap();
                // rotate the image
                img = match rotation {
                    90 => img.rotate90(),
                    180 => img.rotate180(),
                    270 => img.rotate270(),
                    _ => panic!(),
                };
                // write the result to a file like object
                let mut img_file = Cursor::new(Vec::new());
                write_buffer_with_format(
                    &mut img_file,
                    img.as_bytes(),
                    img.width(),
                    img.height(),
                    img.color(),
                    format,
                )
                .unwrap();
                img_file.seek(SeekFrom::Start(0)).unwrap();
                let img_vec = img_file.into_inner();
                // Send the bytes of the image file to discord
                reaction
                    .channel_id
                    .send_files(
                        &ctx.http,
                        vec![(img_vec.as_slice(), format!("ROTATED_{}", img_name).as_str())],
                        |m| {
                            m.content(format!("Rotated image from {}", author))
                                .allowed_mentions(|m| m.empty_users())
                        },
                    )
                    .await
                    .unwrap();
            }
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
