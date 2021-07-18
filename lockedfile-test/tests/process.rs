use std::path::PathBuf;
use std::process::Stdio;
use tokio::process::{Command, Child, ChildStdin, ChildStdout};
use tokio::io::{
    AsyncBufReadExt,
    AsyncWriteExt,
    BufReader,
};
use std::time::Duration;
use std::sync::Arc;
use tokio::sync::Mutex;
use lockedfile_test::*;
use tracing::instrument;
use tracing_subscriber::prelude::*;
mod common;

fn init_tracing() -> tracing_core::dispatcher::DefaultGuard {
    tracing_subscriber::fmt()
        .with_env_filter("process=trace,lockedfile-test=trace")
        .with_ansi(true)
        .pretty()
        .finish()
        .set_default()
}

struct TestProcess {
    name: String,
    child: Child,
    stdin: Arc<Mutex<ChildStdin>>,
    stdout: Arc<Mutex<BufReader<ChildStdout>>>,
}

impl std::fmt::Debug for TestProcess {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> Result<(), std::fmt::Error> {
        write!(f, "TestProcess(\"{}\")", self.name)
    }
}

impl TestProcess {
    #[instrument]
    async fn quit(mut self) {
        let mut stdin = self.stdin.lock().await;

        tracing::info!("Send a quit message");

        let req = Message::Quit;
        let mut s = req.to_json_string();
        s.push('\n');

        stdin.write(s.as_bytes()).await.unwrap();
        stdin.flush().await.unwrap();

        let _ = self.child.wait().await.unwrap();

        tracing::info!("{:?} has quit", self);
    }

    async fn send(&mut self, req: Message) {
        let mut s = req.to_json_string();
        s.push('\n');

        let mut stdin = self.stdin.lock().await;
        stdin.write(s.as_bytes()).await.unwrap();
        stdin.flush().await.unwrap();
    }

    async fn receive(&mut self) -> Message {
        let mut buf = String::new();

        let mut stdout = self.stdout.lock().await;
        stdout.read_line(&mut buf).await.unwrap();

        Message::from_str(&buf)
    }

    async fn exec(&mut self, request: Message) {
        tracing::debug!(request = ?request, "Send a message");
        self.send(request.clone()).await;

        let reply = self.receive().await;
        tracing::debug!(reply = ?reply, "Received a message");

        assert_eq!(request, reply);
    }

    #[instrument]
    async fn create_exclusive(&mut self, path: PathBuf) {
        let req = Message::create_exclusive(path);

        self.exec(req).await;
    }

    #[instrument]
    async fn file_length(&mut self, expected: u64) {
        let req = Message::file_length(expected);
        
        self.exec(req).await;
    }

    #[instrument]
    async fn open_shared(&mut self, path: PathBuf) {
        let req = Message::open_shared(path);

        self.exec(req).await;
    }

    #[instrument]
    async fn read_range(&mut self, start: u64, end: u64) {
        self.exec(Message::read_range(start, end)).await;
    }

    #[instrument]
    async fn write_range(&mut self, start: u64, end: u64) {
        self.exec(Message::write_range(start, end)).await;
    }
}

#[derive(Debug,Clone,Copy)]
struct TestProgram {
    mode: AppMode,
}

impl TestProgram {
    fn new(mode: AppMode) -> Self {
        Self { mode: mode }
    }

    #[instrument]
    fn spawn(&self, name: String) -> TestProcess {
        let mut child = Command::new(env!("CARGO_BIN_EXE_app"))
            .arg(self.mode.to_option_flag())
            .stderr(Stdio::null())
            .stdout(Stdio::piped())
            .stdin(Stdio::piped())
            .spawn()
            .unwrap();

        tracing::info!(name=?name, pid=?child.id(), "Spawned a child process");

        let stdin = child.stdin.take().unwrap();
        let stdout = child.stdout.take().unwrap();

        TestProcess {
            name: name,
            child: child,
            stdin: Arc::new(Mutex::new(stdin)),
            stdout: Arc::new(Mutex::new(BufReader::new(stdout))),
        }
    }
}

#[tokio::test]
async fn exclusive_lock() {
    let _g = init_tracing();

    exclusive_lock_inner().await;
}

#[instrument]
async fn exclusive_lock_inner() {
    let path = common::create_temp_path();
    tracing::debug!("Temporary file path: {:?}", path.as_os_str());

    let testprog = TestProgram::new(AppMode::Std);

    let mut proc = testprog.spawn("Main process".to_owned());

    proc.create_exclusive(path.to_path_buf()).await;
    proc.write_range(0, 1024).await;
    proc.file_length(1024).await;

    let mut another = testprog.spawn("Another process".to_owned());

    tokio::join!(
        async {
            tokio::time::sleep(Duration::from_secs(1)).await;
            proc.quit().await;
        },
        async {
            another.create_exclusive(path.to_path_buf()).await;
            another.file_length(1024).await;
        },
    );

    another.quit().await;
}

#[tokio::test]
async fn open_shared() {
    let _g = init_tracing();

    open_shared_inner().await;
}

#[instrument]
async fn open_shared_inner() {
    let path = common::create_temp_path();
    tracing::debug!("Temporary file path: {:?}", path.as_os_str());

    let prog = TestProgram::new(AppMode::Std);

    {
        let mut proc = prog.spawn("Create file".to_owned());
        proc.create_exclusive(path.to_path_buf()).await;
        proc.write_range(0, 1024).await;
        proc.quit().await;
    }

    let mut proc_a = prog.spawn("Process A".to_owned());
    let mut proc_b = prog.spawn("Process B".to_owned());

    tracing::info!("Open the file with child processes");

    tokio::join! {
        proc_a.open_shared(path.to_path_buf()),
        proc_b.open_shared(path.to_path_buf()),
    };

    tracing::info!("Read the file");

    tokio::join! {
        proc_a.read_range(0, 512),
        proc_b.read_range(256, 1024),
    };

    tokio::join! {
        proc_a.quit(),
        proc_b.quit(),
    };
}
