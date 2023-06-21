use std::{process::{Command,Stdio}, io::{BufReader, BufRead}};

fn main() {

    let mut command = Command::new("python")
        .arg("python-test.py")
        // .stdout(Stdio::piped())
        .spawn()
        .unwrap();

    // let stdout = command.stdout.take().unwrap();

    // let mut bufread = BufReader::new(stdout);
    // let mut buf = String::new();

    // while let Ok(n) = bufread.read_line(&mut buf) {
    //     if n > 0 {
    //         println!("Line: {}", buf.trim());
    //         buf.clear();
    //     } else {
    //         break;
    //     }
    // }
    

}