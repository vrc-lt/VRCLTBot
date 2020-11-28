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
use std::fmt::Display;
use std::io::{self, Read, Write, BufWriter};
use std::process::Command;
use std::env;
use tokio::signal::unix::{signal, SignalKind};
  

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
                    clean_up_tmp_dirs();
                    let _ = download_pdf(attachment_url.to_string()).await;
                    let _ = convert_pdf_to_png();
                    let paths = vec!["./downloaded/result.mp4"];
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
async fn main() -> Result<(), Box<dyn std::error::Error>> {
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

    let mut stream = signal(SignalKind::terminate())?;

    loop {
        stream.recv().await;
    }

}

#[command]
async fn ping(ctx: &Context, msg: &Message) -> CommandResult {
    msg.reply(ctx, "Pong!").await?;

    Ok(())
}

fn clean_up_tmp_dirs(){
    let _ = Command::new("rm")
            .arg("-rf")
            .arg("downloaded")
            .output()
            .expect("failed to execute process").stdout;
    let rm_downloaded = Command::new("mkdir")
            .arg("downloaded")
            .output()
            .expect("failed to execute process").stdout;

    println!("running rm: {:#?}", String::from_utf8(rm_downloaded));
}
fn convert_pdf_to_png(){
    let _ = Command::new("pdftoppm")
            .current_dir("downloaded")
            .arg("-scale-to")
            .arg("1280")
            .arg("-png")
            .arg("downloaded.pdf")
            .arg("image")
            .output()
            .expect("failed to execute process").stdout;
    let exit_status = Command::new("ffmpeg")
            .current_dir("downloaded")
            .arg("-y")
            .arg("-pattern_type")
            .arg("glob")
            .arg("-r")
            .arg("1/2")
            .arg("-i")
            .arg("image-*.png")
            .arg("-c:v")
            .arg("libx264")
            .arg("-r")
            .arg("30")
            .arg("-pix_fmt")
            .arg("yuv420p")
            .arg("result.mp4")
            .output()
            .expect("failed to execute process").stderr;
    println!("running ffmpeg: {:#?}", String::from_utf8(exit_status));
    println!("conversion finished.");
}

async fn download_pdf(url_string: String) -> Result<Bytes, reqwest::Error>{
    let client = reqwest::Client::new();
    let resp = client.get(&url_string)
        .send()
        .await?;
    let res_bytes = resp.bytes().await?;
    println!("pdf file downloaded.");
    write_binary_file(res_bytes.clone()).unwrap();
    return Ok(res_bytes);
}

fn write_binary_file(bytes: Bytes) -> std::io::Result<()>{
    let mut file = BufWriter::new(fs::File::create("./downloaded/downloaded.pdf").unwrap());
    file.write_all(bytes.as_ref())?;
    println!("pdf file wrote.");
    Ok(())
}
