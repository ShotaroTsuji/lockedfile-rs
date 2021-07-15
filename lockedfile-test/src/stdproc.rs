use std::io::BufRead;
use lockedfile_test::Executor;
use lockedfile_test::file::StdFile;

#[tokio::main(flavor = "current_thread")]
async fn main() {
    let mut ex = Executor::<StdFile>::new();
    let stdin = std::io::stdin();
    let mut handle = stdin.lock();

    let mut buf = String::new();

    loop {
        let _ = handle.read_line(&mut buf).unwrap();

        if buf.is_empty() {
            break;
        }

        let result = ex.exec_str(&buf).await;

        match result {
            Ok(rep) => {
                println!("{}", rep.to_json_string());
            },
            Err(e) => {
                eprintln!("{}", e);
                break;
            },
        }
        
        buf.clear();
    }
}
