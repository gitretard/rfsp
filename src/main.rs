mod lib;
use std::{
    env,
    fs::{self, File},
    io::{Read, Write},
    net::{self},
    str, time,convert
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
    let meta = lib::craft_metadata(f);
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
        // test file
        let f = fs::File::open("README.md").unwrap();
        let mut idempt: u32 = 0;
        loop {
            // May be a bit messed up but i just want to get it working. Ill work on it again dont worry
            if let Some(data) = lib::craft_data_packet(idempt, &f) {
                conn.write(&data).unwrap();
                conn.read(&mut buf).unwrap();
                match buf[0] {
                    3 => {
                        println!("Receiver requested cancel!");
                        return;
                    }
                    2 => {
                        println!("Receiver requested resend! resending 1 time");
                        conn.write(&buf).unwrap();
                        continue;
                    }
                    _ => {}
                }
                idempt += 1;
            } else {
                // Returned none. Sending 2
                conn.write(&lib::craft_done_packet()).unwrap();
            }
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
    let mut buf: [u8; 512] = [0; 512];
    for conn in listener.incoming() {
        let mut conn = conn.unwrap(); //gee
        conn.set_read_timeout(Some(time::Duration::from_secs(RW_TIMEOUT)))
            .unwrap();
        conn.set_write_timeout(Some(time::Duration::from_secs(RW_TIMEOUT)))
            .unwrap();
        // read
        match conn.read(&mut buf) {
            Ok(_) => {}
            Err(s) => {
                println!("Error in TcpRead (first packet): {s}");
                continue;
            }
        }
        // IF i have problems. its because how i index
        // Wont do anything with opcode yet
        let arr: [u8; 8] = buf[0..8]
            .try_into()
            .unwrap();
        let filesize: u64 = u64::from_be_bytes(arr);
        let array: [u8; 4] = buf[8..12].try_into().expect("Incorrect slice length");
        let idempt: u32 = u32::from_be_bytes(array);
        if idempt != 0 {
            println!("Invalid idempt value! (must be 0 on metadata handshake)");
            continue;
        }
        let filen = match str::from_utf8(&buf[12..512]) {
            Ok(t) => t,
            Err(e) => {
                println!("Failed to conver [u8] into &str (filen){e}");
                return;
            } // maybe return
        };
        println!("filesize: {filesize}\nfilen: {filen}\nidempt: {idempt}\nAll ok!\nCreating {filen} in the current dir");
        conn.write(&[1]).unwrap();
        // I need a new buffer :(
        let mut cat: [u8;512] = [0;512];
        let mut f = fs::File::create(&filen.replace("\x00","")).unwrap();
        loop {
            conn.read(&mut cat).unwrap(); // Stuck here
            println!("{:?}",cat);
            match cat[8]{
                1 => {println!("Sender requested cancel! cancelling operation");
                println!("Removing file");
                fs::remove_file(&filen);
                continue;
            }
                2 => {}
                _ => {println!("Sender sent unknown opcode. Closing connection");return;}
            }
            // i gotta check the hash damnit
            if lib::hash(&cat[8..512]) != u64::from_be_bytes(cat[0..8].try_into().unwrap()){
                conn.write(&[2;2]).unwrap();
                continue;
            }
            f.write(&cat[14..512]).unwrap();
        }
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
