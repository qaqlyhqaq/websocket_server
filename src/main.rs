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

async fn ws(
    req: HttpRequest,
    body: web::Payload,
    str_data:web::Data<String>,
) -> actix_web::Result<impl Responder> {
    let (response, mut session, mut msg_stream) = actix_ws::handle(&req, body)?;

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
            .app_data("asdf".to_string())
            .wrap(Logger::default())
            .route("/ws", web::get().to(ws))
    })
        .bind("127.0.0.1:8080")?
        .run()
        .await?;

    Ok(())
}
