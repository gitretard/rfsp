mod librecv;
mod libsend;
use std::{
    env,
    io::{Read, Write},
    net::{self},
    str, time,
};

const RW_TIMEOUT: u64 = 30;
const MAX_RETRIES: u32 = 3;

fn send(f: &str) {
    let mut conn = match net::TcpStream::connect("127.0.0.1:5333") {
        Ok(r) => r,
        Err(a) => {
            println!("Error in tcpcon: {}", a);
            return;
        }
    };
    conn.set_read_timeout(Some(time::Duration::from_secs(RW_TIMEOUT)))
        .unwrap(); // Not doing these
    conn.set_write_timeout(Some(time::Duration::from_secs(RW_TIMEOUT)))
        .unwrap();
    let meta = libsend::craft_metadata(f);
    println!("{:?}", meta);
    for _ in 0..MAX_RETRIES {
        match conn.write_all(&meta) {
            Ok(_) => {}
            Err(s) => {
                println!("Send first packet failed: {}", s);
                continue;
            }
        }
        conn.flush().unwrap(); // no i am not making a match statement for this one
                               // Confirm if data has reached
        let mut buf: [u8; 2] = [0; 2];
        for _ in 0..3 {
            let bytes_read: usize = match conn.read(&mut buf) {
                Err(s) => {
                    println!(
                        "Failed to get confirmation (cant read) (Metadata): {s}\nTrying again..."
                    );
                    continue;
                }
                Ok(s) => s,
            };
            if bytes_read == 0 {
                continue;
            }
        }
        if buf[0] < 1 || buf[0] > 3 {
            println!("Invalid status code! The reciever probably isn't a RFSP receiver... Closing the connection (no retries)");
            return;
        }
        match buf[0] {
            // Dont do anything except print yet
            1 => {
                println!("Metadata status packet: 1");
                break;
            }
            2 => {
                println!("Metadata status packet: 2... Retrying");
                continue;
            }
            _ => {}
        }
    }
}

fn recv() {
    let listener = match net::TcpListener::bind("127.0.0.1:5333") {
        Ok(a) => a,
        Err(p) => {
            println!("Error in binding Tcpln: {}", p);
            return;
        }
    };
    let filen:&str;
    for conn in listener.incoming() {
        let mut conn = conn.unwrap(); //gee
        conn.set_read_timeout(Some(time::Duration::from_secs(RW_TIMEOUT))).unwrap();
        conn.set_write_timeout(Some(time::Duration::from_secs(RW_TIMEOUT))).unwrap();
        let mut buf: [u8; 512] = [0; 512];
        // read
        match conn.read(&mut buf) {
            Ok(_) => {}
            Err(s) => {
                println!("Error in TcpRead (first packet): {s}. Cannnot continue");
                return;
            }
        }
        // IF i have problems. its because how i index
        // Wont do anything with opcode yet
        let arr: [u8; 8] = buf[0..8]
            .try_into()
            .expect("Error in converting [u8] into [u8;8]");
        let filesize: u64 = u64::from_be_bytes(arr);
        filen = match str::from_utf8(&buf[8..512]) {
            Ok(t) => t,
            Err(e) => {
                println!("Failed to conver [u8] into &str {e}");
                return;
            } // maybe return
        };
        println!("filesize: {filesize}\nfilen: {filen}");
        conn.write(&[1]).unwrap();
        break;
    }
}

fn main() {
    let a = env::args().nth(3); // cargo run . -- ?
    if let Some(s) = a {
        println!("{s}");
        match s.as_str() {
            "send" => send("src/main.rs"),
            "recv" => recv(),
            _ => {}
        }
    }
}
