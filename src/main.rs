use std::{
    io::{self, Read},
    thread,
    time::Duration,
};

use byteorder::{LittleEndian, ReadBytesExt};
use headless_chrome::LaunchOptionsBuilder;
use roblox_browser::{browser::Browser as RobloxBrowser, stream};
use tiny_http::{Header, Response, Server, StatusCode};

fn main() {
    // Используем переменную окружения PORT для Railway
    let port = std::env::var("PORT").unwrap_or_else(|_| "3000".to_string());
    let addr = format!("0.0.0.0:{}", port);
    
    println!("Starting server on {}", addr);

    let (mut client_stream, server_stream) = stream::stream(4 * 1024 * 1024);
    client_stream.set_read_timeout(Duration::from_secs(15));

    let server = Server::http(&addr).unwrap_or_else(|e| {
        eprintln!("Failed to start server: {}", e);
        std::process::exit(1);
    });

    // Создаем и запускаем браузер
    let launch_options = LaunchOptionsBuilder::default()
        .idle_browser_timeout(Duration::MAX)
        .sandbox(false) // Отключаем sandbox для Railway
        .headless(true)
        .args(vec![
            "--no-sandbox",
            "--disable-gpu",
            "--disable-dev-shm-usage",
            "--disable-setuid-sandbox",
            "--disable-accelerated-2d-canvas",
            "--disable-background-timer-throttling",
            "--disable-backgrounding-occluded-windows",
            "--disable-breakpad",
            "--disable-component-extensions-with-background-pages",
            "--disable-extensions",
            "--disable-features=TranslateUI",
            "--disable-ipc-flooding-protection",
            "--disable-renderer-backgrounding",
            "--enable-features=NetworkService,NetworkServiceInProcess",
            "--force-color-profile=srgb",
            "--hide-scrollbars",
            "--metrics-recording-only",
            "--mute-audio",
            "--no-default-browser-check",
            "--no-first-run",
            "--no-zygote",
        ])
        .build()
        .unwrap_or_else(|e| {
            eprintln!("Failed to build launch options: {}", e);
            std::process::exit(1);
        });

    match RobloxBrowser::start(server_stream, launch_options) {
        Ok(_) => println!("Browser started successfully"),
        Err(e) => {
            eprintln!("Failed to start browser: {}", e);
            std::process::exit(1);
        }
    }

    println!("Server is ready to accept connections");

    for mut req in server.incoming_requests() {
        let mut client_stream = client_stream.clone();

        thread::spawn(move || {
            // Читаем данные из запроса
            let mut reader = req.as_reader();
            let max = match reader.read_u32::<LittleEndian>() {
                Ok(size) => size as usize,
                Err(e) => {
                    eprintln!("Failed to read size: {}", e);
                    let _ = req.respond(Response::new_empty(StatusCode(400)));
                    return;
                }
            };

            // Передаем данные в браузер
            if let Err(e) = io::copy(&mut reader, &mut client_stream) {
                eprintln!("Failed to copy data to browser: {}", e);
                let _ = req.respond(Response::new_empty(StatusCode(500)));
                return;
            }

            // Читаем ответ от браузера
            let mut buf = vec![0; max];
            let amt = if max > 0 {
                match client_stream.read(&mut buf) {
                    Ok(size) => size,
                    Err(e) => {
                        eprintln!("Failed to read response from browser: {}", e);
                        let _ = req.respond(Response::new_empty(StatusCode(500)));
                        return;
                    }
                }
            } else {
                0
            };

            // Отправляем ответ клиенту
            let response = Response::new(
                StatusCode(200),
                vec![
                    Header::from_bytes(&b"Content-Type"[..], &b"application/octet-stream"[..])
                        .unwrap(),
                ],
                &buf[..amt],
                Some(amt),
                None,
            );

            if let Err(e) = req.respond(response) {
                eprintln!("Failed to send response: {}", e);
            }
        });
    }
}
