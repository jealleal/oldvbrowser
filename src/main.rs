use std::{
    io::{Read},
    thread,
    time::Duration,
    sync::{Arc, atomic::{AtomicBool, Ordering}},
};

use byteorder::{LittleEndian, ReadBytesExt};
use headless_chrome::LaunchOptions;
use roblox_browser::{browser::Browser, stream};
use tiny_http::{Header, Method, Response, Server, StatusCode};

fn main() {
    let port = std::env::var("PORT").unwrap_or_else(|_| "3000".to_string());
    let server = Server::http(format!("0.0.0.0:{port}")).unwrap();

    let (mut client_stream, server_stream) = stream::stream(4 * 1024 * 1024);
    client_stream.set_read_timeout(Duration::from_secs(15));

    let browser_ready = Arc::new(AtomicBool::new(false));
    let ready_flag = browser_ready.clone();

    let mut attempts = 0;
    loop {
        attempts += 1;

        let result = Browser::start(
            server_stream.clone(),
            LaunchOptions::default_builder()
                .idle_browser_timeout(Duration::MAX)
                .enable_logging(false)
                .port(Some(9222))
                .sandbox(false)
                .build()
                .unwrap(),
        );

        match result {
            Ok(_browser) => {
                // Считаем браузер готовым сразу после успешного старта
                // Если нужно, тут можно добавить прогрев страницы внутри Browser::start
                ready_flag.store(true, Ordering::SeqCst);
                break;
            }
            Err(_) => {
                if attempts >= 3 {
                    panic!("browser start failed");
                }
                thread::sleep(Duration::from_secs(2));
            }
        }
    }

    for mut req in server.incoming_requests() {
        let mut client_stream = client_stream.clone();
        let ready_flag = browser_ready.clone();

        thread::spawn(move || {
            if req.method() == &Method::Get {
                let _ = req.respond(Response::from_string("OK"));
                return;
            }

            if req.method() != &Method::Post {
                let _ = req.respond(Response::empty(StatusCode(405)));
                return;
            }

            if !ready_flag.load(Ordering::SeqCst) {
                // Если браузер ещё не готов, возвращаем 503
                let _ = req.respond(Response::empty(StatusCode(503)));
                return;
            }

            let mut reader = req.as_reader();

            let max = match reader.read_u32::<LittleEndian>() {
                Ok(v) => v as usize,
                Err(_) => {
                    let _ = req.respond(Response::empty(StatusCode(400)));
                    return;
                }
            };

            if std::io::copy(&mut reader, &mut client_stream).is_err() {
                let _ = req.respond(Response::empty(StatusCode(500)));
                return;
            }

            let mut buf = vec![0u8; max];
            let amt = match client_stream.read(&mut buf) {
                Ok(n) => n,
                Err(_) => {
                    let _ = req.respond(Response::empty(StatusCode(500)));
                    return;
                }
            };

            let _ = req.respond(Response::new(
                StatusCode(200),
                vec![Header::from_bytes("Content-Type", "application/octet-stream").unwrap()],
                &buf[..amt],
                Some(amt),
                None,
            ));
        });
    }
}
