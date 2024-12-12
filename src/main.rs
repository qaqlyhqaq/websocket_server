#![feature(slice_as_array)]
#![feature(mpmc_channel)]
#![feature(pattern)]
#![feature(async_closure)]

use std::collections::HashMap;
use actix_web::{App, HttpRequest, HttpServer, Responder, middleware::Logger, web};
use actix_ws::Message;
use log::log;
use std::io::Read;
use std::str::pattern::Pattern;
use std::sync::Arc;
use std::sync::mpmc::TryRecvError;
use std::sync::mpsc::channel;
use serde_json::{json, Value};

async fn ws(req: HttpRequest, body: web::Payload) -> actix_web::Result<impl Responder> {
    let (response, mut session, mut msg_stream) = actix_ws::handle(&req, body)?;

    actix_web::rt::spawn(async move {
        let (sender, receiver) = channel();

        let mut session_ = session.clone();

        let handle = std::thread::spawn(async move || {
            println!("开始发送指令");
            let mut input_str = String::new();
            let stdin = std::io::stdin();
            loop {
                stdin.read_line(&mut input_str).unwrap();

                match input_str {
                    ref in_str if in_str.starts_with("e") => {
                        sender.send("exit".to_string()).unwrap();
                        unsafe {
                            session_.close(None).await.unwrap();
                        }
                        break;
                    }
                    _ => {
                        //ignore
                    }
                }

                input_str.clear();

                let x = include_str!("../resource/example.1.txt");
                // let x = include_str!("../resource/example.json");
                let value = json!({
                    "data": x,
                });
                let string = serde_json::to_string(&value).unwrap();
                // println!("string:{}", string);
                // sender.send(string).unwrap();
                unsafe {
                    session_.text(string).await.unwrap();
                }
                println!("发送指令");
            }
        });

        drop(handle);



        while let Some(Ok(msg)) = msg_stream.recv().await {
            match msg {
                Message::Ping(bytes) => {
                    // println!("pong: {:?}", String::from_utf8(bytes.to_owned().to_vec()));
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
        println!("websocket 会话已关闭!");
    });

    Ok(response)
}

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    log::log!(
        log::Level::Error,
        "Starting HTTP server at http://localhost:8080"
    );
    println!("Starting HTTP server at http://localhost:8080");

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
