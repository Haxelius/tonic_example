use tonic::{Request, Response, Status, transport::Server};

use file_transfer::file_transfer_server::{FileTransfer, FileTransferServer};
use file_transfer::{FileReply, FileRequest};

pub mod file_transfer {
    include!("../proto/output/file_transfer.rs");
    //tonic::include_proto!("helloworld");
}

#[derive(Debug, Default)]
pub struct MyFileTransfer {}

#[tonic::async_trait]
impl FileTransfer for MyFileTransfer {
    async fn request_file(
        &self,
        request: Request<FileRequest>,
    ) -> Result<Response<FileReply>, Status> {
        let data = std::fs::read(request.into_inner().filename)
            .map_err(|err| Status::from_error(Box::new(err)))?;
        let reply = FileReply { data };
        Ok(Response::new(reply))
    }
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let addr = "[::1]:50051".parse()?;
    let file_request_server = MyFileTransfer::default();

    Server::builder()
        .add_service(FileTransferServer::new(file_request_server))
        .serve(addr)
        .await?;

    Ok(())
}
