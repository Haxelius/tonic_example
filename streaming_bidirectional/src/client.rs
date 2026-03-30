use pb::bi_stream_client::BiStreamClient;
use tokio::{sync::mpsc, task::spawn_blocking};
use tokio_stream::StreamExt;

use crate::pb::TextIntoServer;
pub mod pb {
    tonic::include_proto!("bi_stream");
}

fn input() -> String {
    let mut buf = String::new();
    std::io::stdin().read_line(&mut buf);
    if buf.ends_with('\n') {
        buf.pop();
        if buf.ends_with('\r') {
            buf.pop();
        }
    }
    return buf;
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let dst = "http://[::1]:50051";
    let mut client = BiStreamClient::connect(dst).await?;

    let (sender, receiver) = mpsc::channel(128);
    tokio::spawn(async move {
        loop {
            let user_input = spawn_blocking(input).await.unwrap();
            println!("sending message");
            sender.send(TextIntoServer { text: user_input }).await;
        }
    });
    let to_server_stream = tokio_stream::wrappers::ReceiverStream::new(receiver);
    let mut from_server_stream = client
        .share_text(to_server_stream)
        .await
        .unwrap()
        .into_inner();
    while let Some(message) = from_server_stream.next().await {
        match message {
            Ok(message) => {
                println!("{}", message.text);
            }
            Err(err) => {
                println!("Error: {err}");
            }
        };
    }

    Ok(())
}
