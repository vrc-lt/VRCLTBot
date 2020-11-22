use serenity::async_trait;
use serenity::client::{Client, Context, EventHandler};
use serenity::model::channel::Message;
use serenity::framework::standard::{
    StandardFramework,
    CommandResult,
    macros::{
        command,
        group
    }
};

mod pdf_to_png;

extern crate reqwest;

use bytes::Bytes;

use std::fs;
use std::process::Command;
use std::env;
use std::collections::HashMap;
  

#[group]
#[commands(ping)]
struct General;

struct Handler;

#[async_trait]
impl EventHandler for Handler {
  async fn message(&self, ctx: Context, msg: Message){
      match msg.mentions_me(&ctx).await{
          Ok(true) => {
              if !msg.attachments.is_empty() {
                let attachment_url = &msg.attachments[0].url;
                if let Err(why) = msg.reply(ctx, format!("まだ変換はできません。{}", attachment_url)).await{
                    println!("Error sending message: {:?}", why);
                }

              }else{
                 ();
              }
          }
          _ => ()
      }
  }
}

// #[tokio::main]
// async fn main() {
//     let framework = StandardFramework::new()
//         .configure(|c| c.prefix("~")) // set the bot's prefix to "~"
//         .group(&GENERAL_GROUP);

//     // Login with a bot token from the environment
//     let token = env::var("DISCORD_TOKEN").expect("token");
//     let mut client = Client::builder(token)
//         .event_handler(Handler)
//         .framework(framework)
//         .await
//         .expect("Error creating client");

//     // start listening for events by starting a single shard
//     if let Err(why) = client.start().await {
//         println!("An error occurred while running the client: {:?}", why);
//     }
// }
#[tokio::main]
async fn main() {
    let downloaded = download_pdf("https://cdn.discordapp.com/attachments/779324182639018015/779879506127224832/VRC-LT_7_Opening.pdf".to_string()).await;
    let _ =convert_pdf_to_png();
}

#[command]
async fn ping(ctx: &Context, msg: &Message) -> CommandResult {
    msg.reply(ctx, "Pong!").await?;

    Ok(())
}

fn convert_pdf_to_png(){
    let _ = Command::new("rm")
            .arg("-rf")
            .arg("./downloaded-*")
            .output()
            .expect("failed to execute process").stdout;
    let _ = Command::new("pdftoppm")
            .arg("-png")
            .arg("downloaded.pdf")
            .arg("downloaded-image")
            .output()
            .expect("failed to execute process").stdout;
    let _ = Command::new("ffmpeg")
            .arg("-r")
            .arg("1")
            .arg("-i")
            .arg("downloaded-image-%d.png")
            .arg("-c:v")
            .arg("libx264")
            .arg("-r")
            .arg("30")
            .arg("-pix_fmt")
            .arg("yuv420p")
            .arg("result.mp4")
            .output()
            .expect("failed to execute process").stdout;
}

async fn download_pdf(url_string: String) -> Result<Bytes, reqwest::Error>{
    let client = reqwest::Client::new();
    let resp = client.get(&url_string)
        .send()
        .await?;
    let res_bytes = resp.bytes().await?;
    fs::write("downloaded.pdf", res_bytes.to_vec()).unwrap();
    return Ok(res_bytes);
}
