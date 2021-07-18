use std::io::BufRead;
use crate::Executor;
use crate::file::*;

#[derive(Debug,Clone,Copy)]
pub enum AppMode {
    Std,
    Tokio,
}

impl AppMode {
    pub fn to_option_flag(&self) -> &'static str {
        match self {
            AppMode::Std => "--std",
            AppMode::Tokio => "--tokio",
        }
    }

    pub fn from_option_flag(s: &str) -> Option<Self> {
        match s {
            "--std" => Some(AppMode::Std),
            "--tokio" => Some(AppMode::Tokio),
            _ => None,
        }
    }
}

pub fn execute_with_args() {
    let flag = std::env::args().nth(1).unwrap();

    match AppMode::from_option_flag(&flag) {
        Some(AppMode::Std) => execute_std(),
        Some(AppMode::Tokio) => execute_tokio(),
        None => panic!(),
    }
}

pub fn execute_std() {
    let executor = Executor::<StdFile>::new();
    tokio::runtime::Builder::new_current_thread()
        .enable_all()
        .build()
        .unwrap()
        .block_on(async move { repl(executor).await; })
}

pub fn execute_tokio() {
    let executor = Executor::<TokioFile>::new();
    tokio::runtime::Builder::new_multi_thread()
        .enable_all()
        .worker_threads(4)
        .build()
        .unwrap()
        .block_on(async move { repl(executor).await; })
}

pub async fn repl<F: File>(mut executor: Executor<F>) {
    let stdin = std::io::stdin();
    let mut handle = stdin.lock();
    let mut buf = String::new();

    loop {
        let _ = handle.read_line(&mut buf).unwrap();
        if buf.is_empty() {
            break;
        }

        match executor.exec_str(&buf).await {
            Ok(rep) => {
                println!("{}", rep.to_json_string());
                buf.clear();
            },
            Err(e) => {
                eprintln!("{}", e);
                break;
            },
        }
    }
}
