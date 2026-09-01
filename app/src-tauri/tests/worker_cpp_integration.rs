use std::env;
use std::process::{Child, Command};
use std::thread;
use std::time::{Duration, Instant};

use cliplingo_lib::core::WorkerMessage;
use cliplingo_lib::platform::windows::{WorkerPipeClient, WorkerPipeError};

struct WorkerProcess {
    child: Child,
}

impl WorkerProcess {
    fn spawn() -> Self {
        let executable = env::var_os("CLIPLINGO_WORKER_EXE")
            .expect("CLIPLINGO_WORKER_EXE must point to the deterministic worker executable");
        let child = Command::new(executable)
            .spawn()
            .expect("failed to start deterministic C++ worker");
        Self { child }
    }

    fn wait(mut self) {
        let status = self
            .child
            .wait()
            .expect("failed to wait for deterministic C++ worker");
        assert!(status.success(), "worker exited with {status}");
    }
}

impl Drop for WorkerProcess {
    fn drop(&mut self) {
        if matches!(self.child.try_wait(), Ok(None)) {
            let _ = self.child.kill();
            let _ = self.child.wait();
        }
    }
}

fn connect_with_deadline(deadline: Instant) -> WorkerPipeClient {
    loop {
        match WorkerPipeClient::connect_default() {
            Ok(client) => return client,
            Err(WorkerPipeError::Connect(_)) if Instant::now() < deadline => {
                thread::sleep(Duration::from_millis(20));
            }
            Err(error) => panic!("worker pipe connection failed: {error:?}"),
        }
    }
}

fn assert_translation(response: WorkerMessage, request_id: u64, expected: &str) {
    match response {
        WorkerMessage::TranslateResponse {
            request_id: actual_id,
            text,
        } => {
            assert_eq!(actual_id, request_id);
            assert_eq!(text, expected);
        }
        other => panic!("unexpected worker response: {other:?}"),
    }
}

#[test]
#[ignore = "requires CLIPLINGO_WORKER_EXE built by the Windows CI worker step"]
fn cpp_worker_round_trips_protocol_v1_over_named_pipe() {
    let worker = WorkerProcess::spawn();
    let mut client = connect_with_deadline(Instant::now() + Duration::from_secs(5));

    let first = client.translate(101, "こんにちは dunia 🌏").unwrap();
    assert_translation(first, 101, "[FAKE] こんにちは dunia 🌏");

    let second = client.translate(102, "selamat malam").unwrap();
    assert_translation(second, 102, "[FAKE] selamat malam");

    drop(client);
    worker.wait();
}
