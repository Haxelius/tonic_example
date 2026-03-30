use std::{io::Write, sync::OnceLock};

use file_transfer::file_transfer_client::FileTransferClient;
use tonic::transport::Channel;

use crate::file_transfer::{FileRequest, NoFileRequest};
pub mod file_transfer {
    include!("../proto/output/file_transfer.rs");
}

async fn cat_remote(filename: impl Into<String>) -> Result<(), Box<dyn std::error::Error>> {
    let mut client = FileTransferClient::connect("http://[::1]:50051").await?;

    let request = tonic::Request::new(FileRequest {
        filename: filename.into(),
    });

    let response = client.request_file(request).await?;

    let data_vec = response.into_inner().data;
    /*let data: String = data_vec
    .iter()
    .map(|byte| *byte as char)
    .collect::<String>();*/
    std::io::stdout().write_all((*data_vec).into())?;

    //println!("{:?}", data);

    Ok(())
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let mut client = FileTransferClient::connect("http://[::1]:50051").await?;
    loop {
        let mut input = String::new();
        println!("enter file to view: ");
        std::io::stdin().read_line(&mut input)?;
        for _ in 0..2 {
            input.pop();
        }
        println!("{:?}", input);

        match input.as_str() {
            "" => {
                let response = client.default_file(NoFileRequest {}).await?;

                let data_vec = response.into_inner().data;

                std::io::stdout().write_all((*data_vec).into())?;
            }
            _ => match cat_remote(input).await {
                Ok(_) => (),
                Err(err) => println!("{}", err),
            },
        };
    }
    Ok(())
}
