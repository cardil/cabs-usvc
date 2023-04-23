#[macro_use]
extern crate lazy_static;

mod app;
mod drivers;
mod support;

use actix_web::{
    dev::ServerHandle,
    HttpServer,
};
use tokio::sync::mpsc;

use app::{
    config::setup_logger,
    config::State,
};

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    let state = State::default();

    setup_logger(&state.config.environment);

    let port = app::config::get_port();

    // Create the HTTP server
    let serv = HttpServer::new(move || app::create(state.clone()))
        .bind(("127.0.0.1", port))?
        .run();

    let hnd = serv.handle();

    // Send the server handle to the test
    unsafe {
        if let Some(tx) = &TX {
            tx.send(hnd).await.unwrap();
        }
    }

    // Run the server
    serv.await
}

static mut TX: Option<mpsc::Sender<ServerHandle>> = None;

#[cfg(test)]
mod tests {
    use std::{
        io,
        task::Poll,
        thread::sleep,
    };

    use actix_web::{
        error,
        Result,
    };

    use super::*;

    #[test_log::test(actix_web::test)]
    async fn server_boots() -> io::Result<()> {
        let port = portpicker::pick_unused_port().expect("No free ports");
        std::env::set_var("PORT", format!("{}", port));

        // Create a channel to send the server handle to the test
        let rx = create_server_handle_channel();

        let handle = tokio::task::spawn_blocking(move || main());

        let hnd = receive_server_handle(rx).await;

        tokio::task::spawn_blocking(move || {
            // Await the server to be ready
            await_health(port)
        })
        .await??;

        hnd.stop(true).await;

        assert!(handle.await.unwrap().is_ok());

        Ok(())
    }

    async fn receive_server_handle(
        mut rx: mpsc::Receiver<ServerHandle>,
    ) -> ServerHandle {
        let h = rx.recv().await.unwrap();
        unsafe {
            TX = None;
        }
        h
    }

    fn create_server_handle_channel() -> mpsc::Receiver<ServerHandle> {
        let (tx, rx): (
            mpsc::Sender<ServerHandle>,
            mpsc::Receiver<ServerHandle>,
        ) = mpsc::channel(1);
        unsafe {
            TX = Some(tx);
        }
        rx
    }

    fn await_health(port: u16) -> io::Result<()> {
        let times = Times {
            max:  60,
            step: tokio::time::Duration::from_secs(1),
        };
        await_for("health", times, || poll_health(port))
    }

    struct Times {
        max:  u32,
        step: tokio::time::Duration,
    }

    fn await_for(
        desc: &str,
        times: Times,
        f: impl Fn() -> Poll<()>,
    ) -> io::Result<()> {
        let mut count = 0;
        loop {
            match f() {
                Poll::Ready(()) => return Ok(()),
                Poll::Pending => {
                    if count >= times.max {
                        return Err(io::Error::new(
                            io::ErrorKind::Other,
                            format!("Timed out waiting for {}", desc),
                        ));
                    }
                    count += 1;
                    sleep(times.step)
                }
            };
        }
    }

    fn poll_health(port: u16) -> Poll<()> {
        get_health(port)
            .map(|_| Poll::Ready(()))
            .unwrap_or(Poll::Pending)
    }

    fn get_health(port: u16) -> Result<()> {
        let url = format!("http://localhost:{}/health/ready", port);
        let client = reqwest::blocking::Client::new();
        log::info!("Checking health: {}", url);
        let res = client
            .get(&url)
            .send()
            .map_err(error::ErrorInternalServerError)?;
        log::info!("Health check: {}", res.status());
        if res.status().is_success() {
            Ok(())
        } else {
            Err(error::ErrorInternalServerError("Health check failed"))
        }
    }
}
