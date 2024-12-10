#![feature(slice_as_array)]

use std::io::Read;
use actix_web::{middleware::Logger, web, App, HttpRequest, HttpServer, Responder};
use actix_ws::Message;
use futures_util::stream::StreamExt;
use log::log;

async fn ws(req: HttpRequest, body: web::Payload) -> actix_web::Result<impl Responder> {
    let (response, mut session, mut msg_stream) = actix_ws::handle(&req, body)?;

    actix_web::rt::spawn(async move {
        while let Some(Ok(msg)) = msg_stream.next().await {
            match msg {
                Message::Ping(bytes) => {
                    println!("pong: {:?}", String::from_utf8(bytes.to_owned().to_vec()));
                    if session.pong(&bytes).await.is_err() {
                        eprintln!("Session disconnected");
                        return;
                    }
                }
                Message::Text(msg) => println!("Got text: {msg}"),
                _ => break,
            }
        }

        let _ = session.close(None).await;
    });

    Ok(response)
}

#[actix_web::main]
async fn main() -> std::io::Result<()> {

    log::log!(log::Level::Error, "Starting HTTP server at http://localhost:8080");
    println!( "Starting HTTP server at http://localhost:8080");

    HttpServer::new(move || {
        App::new()
            .wrap(Logger::default())
            .route("/ws", web::get().to(ws))
    })
        .bind("127.0.0.1:8080")?
        .run()
        .await?;

    Ok(())
}