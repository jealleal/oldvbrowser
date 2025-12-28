use std::{
    io::{Read},
    thread,
    time::Duration,
};

use headless_chrome::LaunchOptions;
use roblox_browser::{browser::Browser, stream};
use tiny_http::{Header, Method, Response, Server, StatusCode};

fn main() {
    let (mut client_stream, server_stream) = stream::stream(4 * 1024 * 1024);
    client_stream.set_read_timeout(Duration::from_secs(15));

    let server = Server::http("0.0.0.0:3000").unwrap();

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

        if result.is_ok() {
            break;
        }

        if attempts >= 3 {
            panic!("browser start failed");
        }

        thread::sleep(Duration::from_secs(2));
    }

    for mut req in server.incoming_requests() {
        let mut client_stream = client_stream.clone();

        thread::spawn(move || {
            if req.method() == &Method::Get {
                let _ = req.respond(Response::from_string("OK"));
                return;
            }

            if req.method() != &Method::Post {
                let _ = req.respond(Response::empty(StatusCode(405)));
                return;
            }

            let mut body = Vec::new();
            if req.as_reader().read_to_end(&mut body).is_err() {
                let _ = req.respond(Response::empty(StatusCode(400)));
                return;
            }

            if client_stream.write_all(&body).is_err() {
                let _ = req.respond(Response::empty(StatusCode(500)));
                return;
            }

            let mut response = vec![0u8; 4 * 1024 * 1024];
            let size = match client_stream.read(&mut response) {
                Ok(n) => n,
                Err(_) => {
                    let _ = req.respond(Response::empty(StatusCode(500)));
                    return;
                }
            };

            let _ = req.respond(Response::new(
                StatusCode(200),
                vec![Header::from_bytes("Content-Type", "application/octet-stream").unwrap()],
                &response[..size],
                Some(size),
                None,
            ));
        });
    }
}
