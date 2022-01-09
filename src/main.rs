use serenity::async_trait;
use serenity::client::{Client, Context, EventHandler};
use serenity::framework::standard::{
    macros::{command, group},
    CommandResult, StandardFramework,
};
use serenity::model::channel::Message;

use temp_dir::TempDir;

extern crate reqwest;

use bytes::Bytes;

use std::fs;
use std::path::Path;

use std::env;
use std::io::{BufWriter, Write};
use std::process::Command;

#[group]
#[commands(ping)]
struct General;

struct Handler;

#[async_trait]
impl EventHandler for Handler {
    async fn message(&self, ctx: Context, msg: Message) {
        if let Ok(true) = msg.mentions_me(&ctx).await {
            if msg.attachments.is_empty() {
                msg.reply(ctx, "変換したいPDFファイルを添付してください")
                    .await
                    .expect("reply failed");
                return;
            }
            let attachment_url = &msg.attachments[0].url;
            if (!attachment_url.ends_with("pdf")) {
                msg.reply(ctx, "PDFファイルを添付してください")
                    .await
                    .expect("reply failed");
                return;
            }

            let dir = TempDir::new().expect("create temp directory failed");
            let pdf = dir.path().join("download.pdf");
            let _ = download_pdf(attachment_url.to_string(), &pdf).await;
            let _ = convert_pdf_to_png(&pdf);
            let mp4 = dir.path().join("result.mp4");
            println!("post file...");
            let _ = msg
                .channel_id
                .send_files(&ctx, vec![&mp4], |m| m.content("変換しました"))
                .await
                .unwrap();
            println!("files sent.");

            println!("creanup temp dir.");
            drop(dir);
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
    let shard_manager = client.shard_manager.clone();
    tokio::spawn(async move {
        tokio::signal::ctrl_c()
            .await
            .expect("Could not register ctrl+c handler");
        shard_manager.lock().await.shutdown_all().await;
    });
}

#[command]
async fn ping(ctx: &Context, msg: &Message) -> CommandResult {
    msg.reply(ctx, "Pong!").await?;

    Ok(())
}

fn convert_pdf_to_png(pdf: &Path) {
    let dir = pdf.parent().unwrap();
    let _ = Command::new("pdftoppm")
        .current_dir(dir)
        .arg("-scale-to-x")
        .arg("1280")
        .arg("-scale-to-y")
        .arg("720")
        .arg("-png")
        .arg("downloaded.pdf")
        .arg("image")
        .output()
        .expect("failed to execute process")
        .stdout;
    let exit_status = Command::new("ffmpeg")
        .current_dir(dir)
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
        .expect("failed to execute process")
        .stderr;
    println!("running ffmpeg: {:#?}", String::from_utf8(exit_status));
    println!("conversion finished.");
}

async fn download_pdf(url_string: String, fname: &Path) -> Result<Bytes, reqwest::Error> {
    let client = reqwest::Client::new();
    let resp = client.get(&url_string).send().await?;
    let res_bytes = resp.bytes().await?;
    println!("pdf file downloaded.");
    write_binary_file(fname, res_bytes.clone()).unwrap();
    Ok(res_bytes)
}

fn write_binary_file(fname: &Path, bytes: Bytes) -> std::io::Result<()> {
    let file = fs::File::create(fname)
        .unwrap_or_else(|_| panic!("create binary file failed: {}", fname.to_str().unwrap()));
    let mut file = BufWriter::new(file);
    file.write_all(bytes.as_ref())?;
    println!("pdf file wrote.");
    Ok(())
}
