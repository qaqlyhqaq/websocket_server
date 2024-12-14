#![feature(slice_as_array)]
#![feature(mpmc_channel)]
#![feature(pattern)]
#![feature(async_closure)]
#![feature(duration_constructors)]

use actix_web::rt::Runtime;
use actix_web::{App, HttpRequest, HttpServer, Responder, middleware::Logger, web};
use actix_ws::Message;
use futures_util::StreamExt;
use log::log;
use serde_json::{Value, json};
use std::collections::HashMap;
use std::io::Read;
use std::ops::Deref;
use std::str::pattern::Pattern;
use std::sync::Arc;
use std::sync::mpmc::TryRecvError;
use std::sync::mpsc::channel;
use std::thread;
use std::time::Duration;
use tokio::time::sleep;

async fn ws(req: HttpRequest, body: web::Payload) -> actix_web::Result<impl Responder> {
    let (response, mut session, mut msg_stream) = actix_ws::handle(&req, body)?;

    // let (send,recv) = channel();

    let handle1 = actix_web::rt::spawn(async move {
        loop {
            tokio::select! {
                _ = sleep(Duration::from_secs(5)) => {
                }
                msg = msg_stream.next() => {
                            match msg.unwrap() {
                        Ok(Message::Ping(bytes)) => {
                            if session.clone().pong(&bytes).await.is_err() {
                                return;
                            }
                        }

                        Ok(Message::Text(msg)) => println!("Got text: {msg}"),
                        _ => break,
                    }
                    }
            }
        }

        // while let Some(Ok(msg)) = msg_stream.next().await {
        //     match msg {
        //         Message::Ping(bytes) => {
        //             if session.clone().pong(&bytes).await.is_err() {
        //                 return;
        //             }
        //         }
        //
        //         Message::Text(msg) => println!("Got text: {msg}"),
        //         _ => break,
        //     }
        // }

        session.close(None).await.unwrap();
    });

    Ok(response)
}

#[actix_web::main]
#[log_lib::log_handler]
async fn main() -> std::io::Result<()> {
    log::log!(
        log::Level::Error,
        "Starting HTTP server at http://localhost:8080"
    );

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
