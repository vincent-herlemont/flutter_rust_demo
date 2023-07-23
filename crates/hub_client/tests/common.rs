#![cfg(unix)]

use config::Config;
use nix::sys::signal::{kill, Signal};
use nix::unistd::Pid;
use serial_test::file_serial;
use std::thread::JoinHandle;
use std::time::Duration;
use std::{process, thread};

#[test]
#[file_serial]
#[cfg(feature = "local_supabase")]
fn test_graceful_shutdown() {
    let start_handle = start();
    stop(start_handle);
}

fn start() -> JoinHandle<()> {
    // spawn the server in a new thread
    let handle = thread::spawn(|| {
        let config = Config::new(".env.test");
        let rt = tokio::runtime::Runtime::new().unwrap();
        rt.block_on(async { hub_client::start(config).await })
            .unwrap()
    });

    // let server get up and running
    thread::sleep(Duration::from_secs(1));

    handle
}

fn stop(start_handle: JoinHandle<()>) {
    // send SIGTERM signal to this process, which should trigger graceful shutdown
    let pid = Pid::from_raw(process::id() as i32);
    assert!(kill(pid, Signal::SIGTERM).is_ok());
    // ensure server thread finishes
    assert!(start_handle.join().is_ok());
}
