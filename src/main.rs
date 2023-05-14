mod lib;
use std::{
    env, fs,
    io::{Read, Write},
    net::{self},
    str, time,
};
// May be a bit messed up but i just want to get it working. Ill work on it again dont worry
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
        conn.flush().unwrap(); // no i am not making a match statement for this on                              
        // Confirm if data has reached
        let mut buf: [u8; 2] = [0; 2];
        match conn.read(&mut buf) {
            Err(s) => {
                println!(
                    "Failed to get confirmation (cant read reply) (Metadata): {s}\nClosing connection"
                );
                continue;
            }
            Ok(s) => s,
        };
        if buf[0] < 1 || buf[0] > 3 { // rguh98rw0
            println!("Invalid status code! The reciever probably isn't a RFSP receiver... Closing the connection (no retries)");
            return;
        }
        let f = fs::File::open(&f).unwrap();
        let mut idempt: u32 = 0;
        loop {
            idempt += 1;
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
                // Check if finished (plaster fix)
                if data[9] == 2{
                    println!("Finished sending the file!");
                    return;   
                }
                idempt += 1;
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
        let arr: [u8; 8] = buf[0..8].try_into().unwrap();
        let filesize: u64 = u64::from_be_bytes(arr);
        let array: [u8; 4] = buf[8..12].try_into().expect("Incorrect slice length");
        let mut idempt: u32 = u32::from_be_bytes(array);
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
        println!("filesize: {filesize}\nfilen: {filen}\nidempt: {idempt}\n\nCreating {filen} in the current dir");
        conn.write(&[1]).unwrap();
        // I need a new buffer :(
        let mut cat: [u8; 512] = [0; 512];
        let mut f = fs::File::create(&filen.replace("\x00", "")).unwrap();
        loop {
            conn.read(&mut cat).unwrap(); // Stuck here
            match cat[9] {
                2 => {}
                1 => {
                    println!("Sender requested cancel! cancelling operation");
                    println!("Removing file");
                    fs::remove_file(&filen).unwrap();
                    continue;
                }
                0 => {} // Do nothing
                _ => {
                    println!("Sender sent unknown opcode. Closing connection");
                    return;
                }
            }
            if idempt >= u32::from_be_bytes(cat[10..14].try_into().unwrap()) {
                println!("local idempt >= remote idempt. Closing connection and deleteing file");
                fs::remove_file(&filen.replace("\x00", "")).unwrap();
                conn.write(&[3; 10]).unwrap();
                break;
            }
            idempt += 1;
            if lib::hash(&cat[8..512]) != u64::from_be_bytes(cat[0..8].try_into().unwrap()) {
                conn.write(&[3; 10]).unwrap();
                println!("Cancel: incorrect hash\nNote: This behavior will be changed soon");
                println!(
                    "Local hash: {}\nSender hash: {}\n",
                    lib::hash(&cat[8..512]),
                    u64::from_be_bytes(cat[0..8].try_into().unwrap())
                );
                break;
            }
            f.write(&cat[14..cat.len()]).unwrap();
            conn.write(&[1; 2]).unwrap();
            if cat[9] == 2{
                println!("Finished!");
                break;
            }
        }
        println!("\nAwaiting other connection\n")
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
