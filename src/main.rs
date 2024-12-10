#![feature(slice_as_array)]
#![feature(mpmc_channel)]
use actix_web::{App, HttpRequest, HttpServer, Responder, middleware::Logger, web};
use actix_ws::Message;
use futures_util::stream::StreamExt;
use log::log;
use std::io::Read;
use std::sync::mpmc::TryRecvError;
use std::sync::mpsc::channel;

async fn ws(req: HttpRequest, body: web::Payload) -> actix_web::Result<impl Responder> {
    let (response, mut session, mut msg_stream) = actix_ws::handle(&req, body)?;

    actix_web::rt::spawn(async move {
        let (sender, receiver) = channel();

        let _ = std::thread::spawn(move || {
            loop {
                println!("请输入字母或数字后回车："); //1
                let mut input_str = String::new(); //2
                let stdin = std::io::stdin(); //3
                stdin.read_line(&mut input_str).unwrap(); //4
                println!("您的输入是：{}", input_str); //5
                sender.send(input_str).unwrap();
            }
        })
        .thread();

        while let Some(Ok(msg)) = msg_stream.next().await {
            match receiver.try_recv() {
                Ok(_) => {
                    //发送消息
                    session.text("发送消息 !").await.unwrap();
                }
                Err(_) => {}
            }
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
