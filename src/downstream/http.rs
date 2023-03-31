
use std::sync::Arc;
use std::net::SocketAddr;

use std::sync::mpsc::Sender;
use tokio::io::{
    BufReader,
    AsyncRead,
    AsyncBufReadExt,
};
use tokio::net::{TcpListener, TcpStream};
use async_trait::async_trait;
use bytecodec::bytes::RemainingBytesDecoder;
use bytecodec::io::IoDecodeExt;
use httpcodec::{BodyDecoder, HttpVersion, RequestDecoder};
use tiny_http::{Server, Response};

use super::Service;
use crate::types::{Message, SafeMutex};
pub struct Http {}

#[async_trait]
impl Service for Http {
    /// starts a TcpListener that handles requests asynchronously
    async fn start(id: u32, relay_channel: Sender<Message>) -> Result<(), Box<dyn std::error::Error>> {
        // Construct our SocketAddr to listen on...
        let addr = SocketAddr::from(([127, 0, 0, 1], 5000));
        let server = Server::http(addr.to_string()).unwrap();
        
        tokio::spawn( async move {
            for request in server.incoming_requests() {
                println!("{:?}",&request);
                handle_request(request, relay_channel.clone());
            }
        //     tracing::info!("HTTP listening on {:?}", addr);
        //     loop {
        //         match listener.accept().await {
        //             Ok((stream, socket_addr)) => {
        //                 tracing::info!("Incoming connection at {:?}", socket_addr);
        //                 handle_request(stream, relay_channel.clone());
        //             },
        //             Err(e) => tracing::error!("Failed to accept incoming connection: {:?}", e)
        //         }
        //     }
        });
        Ok(())
    }
}

fn handle_request(req: tiny_http::Request, sender: Sender<Message>) {
    tokio::spawn(async move {
        let _ = sender.send(Message {
            // socket_writer: socket_writer.clone(),
            request: req
        });
    });


}
// fn handle_request_(stream: TcpStream, sender: Sender<Message>) {
//     let mut decoder =
//     RequestDecoder::<BodyDecoder<RemainingBytesDecoder>>::default();
//     tokio::spawn(async move {
//         let (socket_reader, socket_writer) = stream.into_split();
//         let socket_writer = Arc::new(SafeMutex::new(socket_writer));
//         let mut reader = BufReader::new(socket_reader);
//         loop {
//             let mut buf = Vec::new();
//             match reader.read_until(b'', &mut buf).await {
//                 Ok(_) => {
//                     let request= decoder.decode_exact::<&[u8]>(buf.as_ref());
//                     tracing::info!("MSG: {:?}", request);
//                     let _ = sender.send(Message {
//                         socket_writer: socket_writer.clone(),
//                         payload: buf
//                     }).await;
                    
//                 },
//                 Err(e) => {
//                     tracing::error!("Error reading downstream message: {:?}", e);
//                 }
//             }
//         }
//     });
// }
