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


extern crate reqwest;

use bytes::Bytes;

use std::fs;
use std::process::Command;
use std::env;
  

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
                if(attachment_url.to_string().ends_with(".pdf")){
                    let _ = download_pdf(attachment_url.to_string());
                    let _ = convert_pdf_to_png();
                    let paths = vec!["./result.mp4"];
                    println!("post file...");
                    let _ = msg.channel_id.send_files(&ctx, paths, |m| m.content("変換しました")).await.unwrap();
                    println!("files sent.");
                }else{
                    if let Err(why) = msg.reply(ctx, format!("まだ変換はできません。{}", attachment_url)).await{
                        println!("Error sending message: {:?}", why);
                    }
                }

              }else{
                 ();
              }
          }
          _ => ()
      }
  }
}

#[tokio::main]
async fn main() {
    let framework = StandardFramework::new()
        .configure(|c| c.prefix("~")) // set the bot's prefix to "~"
        .group(&GENERAL_GROUP);

    // Login with a bot token from the environment
    let token = env::var("DISCORD_TOKEN").expect("token");
    let mut client = Client::builder(token)
        .event_handler(Handler)
        .framework(framework)
        .await
        .expect("Error creating client");

    // start listening for events by starting a single shard
    if let Err(why) = client.start().await {
        println!("An error occurred while running the client: {:?}", why);
    }
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
    println!("conversion finished.");
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
