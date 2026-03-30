pub mod pb {
    include!("../proto/output/number_stream.rs");
}
use pb::number_stream_client::NumberStreamClient;
use tokio_stream::StreamExt;
use tonic::Request;

use crate::pb::{
    NumberStreamRequest,
    number::{SecondsPassedMessage, Type::*, UserInputMessage},
};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let mut client = NumberStreamClient::connect("http://[::1]:50051")
        .await
        .unwrap();

    let request = Request::new(NumberStreamRequest {});

    let mut stream = client
        .number_stream_rpc(request)
        .await
        .unwrap()
        .into_inner();

    println!("before while");
    while let Some(number) = stream.next().await {
        //println!("recieved {:?}", number);
        match number {
            Ok(num) => match num.r#type {
                None => println!("idk"),
                Some(number_type) => match number_type {
                    SecondsPassed(SecondsPassedMessage { number }) => {
                        println!("{number}s passed");
                    }
                    UserInput(UserInputMessage { number }) => {
                        println!("the user typed: {number}");
                    }
                },
            },
            Err(err) => println!("err: {err}"),
        }
    }
    println!("no next");
    Ok(())
}
