use std::io::BufRead;
use lockedfile_test::Executor;

fn main() {
    let mut ex = Executor::new();
    let stdin = std::io::stdin();
    let mut handle = stdin.lock();

    let mut buf = String::new();

    loop {
        eprintln!("Enter the loop");
        let _ = handle.read_line(&mut buf).unwrap();
        eprintln!("read_line: {}", buf);
        eprintln!("buf.len() = {}", buf.len());

        if buf.is_empty() {
            break;
        }

        let result = ex.exec_str(&buf);

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
