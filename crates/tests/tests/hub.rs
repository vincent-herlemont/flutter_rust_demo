#![cfg(unix)]
use hyper::{Client, Uri};
use nix::sys::signal::{kill, Signal};
use nix::unistd::Pid;
use serial_test::serial;
use std::process;
use std::thread;
use tokio::time::{sleep, Duration};

#[tokio::test]
#[serial]
async fn test_hello_word() {
    std::env::set_var("RUST_LOG", "info");
    let port: u16 = 3033;

    // spawn the server in a new thread
    let handle = tokio::spawn(async move {
        let mut config = hub::Config::new().unwrap();
        config.set_port(port);
        hub::start(&config).await.unwrap()
    });

    // let server get up and running
    sleep(Duration::from_secs(1)).await;

    let uri = Uri::builder()
        .scheme("http")
        .authority(format!("127.0.0.1:{}", port).as_str())
        .path_and_query("/http")
        .build()
        .unwrap();

    println!("uri: {}", uri);
    let res = Client::new().get(uri).await.unwrap();

    assert_eq!(res.status(), 200);

    // send SIGTERM signal to this process, which should trigger graceful shutdown
    let pid = Pid::from_raw(process::id() as i32);
    assert!(kill(pid, Signal::SIGTERM).is_ok());

    // ensure server thread finishes
    assert!(handle.await.is_ok());
}

#[test]
#[serial]
fn test_graceful_shutdown() {
    std::env::set_var("RUST_LOG", "info");

    // spawn the server in a new thread
    let handle = thread::spawn(|| {
        let mut config = hub::Config::new().unwrap();
        config.set_port(3032);
        let rt = tokio::runtime::Runtime::new().unwrap();
        rt.block_on(async { hub::start(&config).await }).unwrap()
    });

    // let server get up and running
    thread::sleep(Duration::from_secs(1));

    // send SIGTERM signal to this process, which should trigger graceful shutdown
    let pid = Pid::from_raw(process::id() as i32);
    assert!(kill(pid, Signal::SIGTERM).is_ok());

    // ensure server thread finishes
    assert!(handle.join().is_ok());
}
