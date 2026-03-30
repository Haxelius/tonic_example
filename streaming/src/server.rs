pub mod pb {
    include!("../proto/output/number_stream.rs");
}

use std::pin::Pin;
use std::time::Duration;

use pb::number_stream_server::{NumberStream, NumberStreamServer};
use pb::{Number, NumberStreamRequest};
use tokio::time::sleep;
use tokio_stream::Stream;
use tokio_stream::wrappers::ReceiverStream;
use tonic::transport::Server;
use tonic::{Request, Response, Status};

use crate::pb::number::{SecondsPassedMessage, Type, UserInputMessage};

type NumberResult<T> = Result<Response<T>, Status>;
type ResponseStream = Pin<Box<dyn Stream<Item = Result<Number, Status>> + Send>>;

#[derive(Debug, Default)]
struct MyNumberStream {}

fn input_number() -> u32 {
    let mut input = String::new();
    loop {
        match {
            println!("input a number to send");
            let res = std::io::stdin().read_line(&mut input);
            if input.ends_with('\n') {
                input.pop();
                if input.ends_with('\r') {
                    input.pop();
                }
            }
            res
        } {
            Ok(_) => match input.parse::<u32>() {
                Ok(num) => {
                    println!("returning {num}");
                    return num;
                }
                Err(_) => println!("[{input}] is not a number"),
            },
            Err(err) => println!("stdin error: {err}"),
        }
    }
}

#[tonic::async_trait]
impl NumberStream for MyNumberStream {
    type NumberStreamRpcStream = ResponseStream;

    async fn number_stream_rpc(
        &self,
        _request: Request<NumberStreamRequest>,
    ) -> NumberResult<Self::NumberStreamRpcStream> {
        let (msg_sender, msg_receiver) = tokio::sync::mpsc::channel(128);

        let tx = msg_sender.clone();
        tokio::spawn(async move {
            loop {
                let number = tokio::task::spawn_blocking(|| input_number())
                    .await
                    .unwrap();
                let num = Number {
                    r#type: Some(Type::UserInput(UserInputMessage { number })),
                };
                let result = Result::<_, Status>::Ok(num);
                match tx.send(result).await {
                    Ok(_) => {}
                    Err(_) => {
                        break;
                    }
                }
            }
            println!("exiting send loop");
        });

        let tx = msg_sender.clone();
        tokio::spawn(async move {
            let mut number = 0;
            loop {
                let result = Result::<_, Status>::Ok(Number {
                    r#type: Some(Type::SecondsPassed(SecondsPassedMessage { number })),
                });
                match tx.send(result).await {
                    Ok(_) => {}
                    Err(_) => {
                        break;
                    }
                }
                number += 1;
                sleep(Duration::from_millis(1000)).await;
            }
            println!("exiting send loop");
        });

        let output_stream = ReceiverStream::new(msg_receiver);
        Ok(Response::new(
            Box::pin(output_stream) as Self::NumberStreamRpcStream
        ))
    }
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let addr = "[::1]:50051".parse()?;
    println!("server running on {}", addr);
    let number_server = MyNumberStream::default();

    let server = Server::builder().add_service(NumberStreamServer::new(number_server));
    server.serve(addr).await?;
    Ok(())
}
