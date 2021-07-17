use std::io::BufRead;
use lockedfile_test::Executor;
use lockedfile_test::file::*;

fn main() {
    let executor = Executor::<StdFile>::new();
    tokio::runtime::Builder::new_current_thread()
        .enable_all()
        .build()
        .unwrap()
        .block_on(async move { repl(executor).await; })
}

async fn repl<F: File>(mut executor: Executor<F>) {
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
