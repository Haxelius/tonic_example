use std::pin::Pin;

use pb::bi_stream_server::{BiStream, BiStreamServer};
use tokio::sync::{Mutex, broadcast, mpsc};
use tokio_stream::{Stream, StreamExt, wrappers::ReceiverStream};
use tonic::{self, Response, Status, transport::Server};

use crate::pb::TextOutOfServer;

pub mod pb {
    tonic::include_proto!("bi_stream");
}

#[derive(Debug, Clone)]
struct TextMessage {
    client_id: u32,
    message: String,
}

struct MyServer {
    sender: broadcast::Sender<TextMessage>,
    receiver: broadcast::Receiver<TextMessage>,
    next_free_id: Mutex<u32>,
}
type TextResponse = Result<TextOutOfServer, Status>;
type ResponseStream = Pin<Box<dyn Stream<Item = TextResponse> + Send>>;

#[tonic::async_trait]
impl BiStream for MyServer {
    type ShareTextStream = ResponseStream;
    async fn share_text(
        &self,
        request: tonic::Request<tonic::Streaming<pb::TextIntoServer>>,
    ) -> Result<Response<ResponseStream>, Status> {
        let mut next_free_id_guard = self.next_free_id.lock().await;
        let my_id = *next_free_id_guard;
        *next_free_id_guard += 1;
        drop(next_free_id_guard);

        let mut in_stream = request.into_inner();

        let sender = self.sender.clone();
        //receive messages from client and send to other clients in server
        tokio::spawn(async move {
            while let Some(received) = in_stream.next().await {
                println!("received something");
                match received {
                    Ok(message) => {
                        let received_text = message.text;
                        println!("client: {my_id} sending {}", received_text);
                        let send_result = sender.send(TextMessage {
                            client_id: my_id,
                            message: received_text,
                        });
                        match send_result {
                            Ok(_) => {}
                            Err(err) => {
                                println!("failed to send {:?}", err.0)
                            }
                        };
                    }
                    Err(err) => {
                        println!("received error: {err}");
                        break;
                    }
                }
            }
        });

        let (output_sender, output_receiver) = mpsc::channel::<TextResponse>(128);
        let mut server_receiver = self.receiver.resubscribe();
        //receive messages from another clients on server and send to this client
        tokio::spawn(async move {
            loop {
                match server_receiver.recv().await {
                    Ok(text_message) => {
                        if text_message.client_id != my_id {
                            let out_message = Ok(TextOutOfServer {
                                text: text_message.message,
                            });
                            match output_sender.send(out_message).await {
                                Ok(_) => {}
                                Err(err) => {
                                    println!("failed to send message to client {my_id}");
                                    break;
                                }
                            }
                        }
                    }
                    Err(err) => {
                        println!("received error: {err}");
                        break;
                    }
                }
            }
        });

        let out_stream = ReceiverStream::new(output_receiver);
        Ok(Response::new(Box::pin(out_stream) as Self::ShareTextStream))
    }
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let addr = "[::1]:50051".parse()?;
    let (sender, receiver) = broadcast::channel::<TextMessage>(128);
    let text_server = MyServer {
        sender,
        receiver,
        next_free_id: Mutex::new(1),
    };

    let server = Server::builder().add_service(BiStreamServer::new(text_server));
    server.serve(addr).await?;
    Ok(())
}
