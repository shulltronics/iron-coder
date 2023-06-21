use std::{process::{Command,Stdio}, io::{BufReader, BufRead}};
use std::io::Read;
fn main() {

    let mut command = Command::new("python")
        .arg("python-test.py")
        .stderr(Stdio::piped())
        .stdout(Stdio::piped())
        .spawn()
        .unwrap();

    let mut stdout = command.stdout.take().unwrap();
    let mut stderr = command.stderr.take().unwrap();

    // let mut bufread = BufReader::new(stdout);
    let mut buf = String::new();

    println!("starting loop");
    // while let Ok(n) = bufread.read_line(&mut buf) {
    //     println!("in loop");
    //     if n > 0 {
    //         println!("Line: {}", buf.trim());
    //         buf.clear();
    //     } else {
    //         break;
    //     }
    // }
    
    loop {
        println!("loop start");
        stdout.read_to_string(&mut buf).unwrap();
        println!("{}", buf);
    }


}