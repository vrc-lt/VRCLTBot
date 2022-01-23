use serenity::async_trait;
use serenity::client::{Client, Context, EventHandler};
use serenity::framework::standard::{
    macros::{command, group},
    CommandResult, StandardFramework,
};
use serenity::model::channel::Message;

extern crate reqwest;

use bytes::Bytes;

use vrcltbot::convert_pdf_to_png;

use std::fs;

use std::env;
use std::io::{BufWriter, Write};
use std::process::Command;
use temp_dir::TempDir;

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

            if (!attachment_url.to_string().ends_with(".pdf")) {
                msg.reply(ctx, "PDFファイルを添付してください")
                    .await
                    .expect("reply failed");
                return;
            }
            if (attachment_url.to_string().ends_with(".pdf")) {
                clean_up_tmp_dirs();
                let dir = TempDir::new().expect("create temp directory failed");
                let pdf_path = dir.path().join(std::path::Path::new("downloaded.pdf"));
                let _ = download_pdf(attachment_url.to_string(), &pdf_path).await;
                let _ = convert_pdf_to_png(&pdf_path);
                let paths = vec![dir.path().join(std::path::Path::new("result.mp4"))];
                println!("post file...");
                let _ = msg
                    .channel_id
                    .send_files(&ctx, &paths, |m| m.content("変換しました"))
                    .await
                    .unwrap();
                println!("files sent.");
                dir.cleanup().unwrap();
            } else if let Err(why) = msg
                .reply(ctx, format!("まだ変換はできません。{}", attachment_url))
                .await
            {
                println!("Error sending message: {:?}", why);
            }
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

fn clean_up_tmp_dirs() {
    let _ = Command::new("rm")
        .arg("-rf")
        .arg("downloaded")
        .output()
        .expect("failed to execute process")
        .stdout;
    let rm_downloaded = Command::new("mkdir")
        .arg("downloaded")
        .output()
        .expect("failed to execute process")
        .stdout;

    println!("running rm: {:#?}", String::from_utf8(rm_downloaded));
}

async fn download_pdf(url_string: String, write_path: &std::path::Path) {
    let client = reqwest::Client::new();
    let resp = client.get(&url_string).send().await.unwrap();
    let res_bytes = resp.bytes().await.unwrap();
    println!("pdf file downloaded.");
    write_binary_file(res_bytes, write_path);
}

fn write_binary_file(bytes: Bytes, write_path: &std::path::Path) {
    let file = fs::File::create(write_path).unwrap();
    let mut file_writer = BufWriter::new(file);
    let _ = file_writer.write_all(bytes.as_ref());
    println!("pdf file wrote.");
}

#[test]
fn test_conversion() {
    let test_pdf_path = std::path::Path::new("./test_data/test.pdf");
    let dir = TempDir::new().expect("create temp directory failed");
    let copied_pdf_path = dir.path().join(std::path::Path::new("test.pdf"));
    std::fs::copy(test_pdf_path, &copied_pdf_path).unwrap();
    let _ = convert_pdf_to_png(&copied_pdf_path);
    std::fs::copy(
        &dir.path().join(std::path::Path::new("result.mp4")),
        std::path::Path::new("./result.mp4"),
    )
    .unwrap();
    dir.cleanup().unwrap();
}
