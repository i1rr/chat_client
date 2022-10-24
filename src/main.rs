use std::io::{self, ErrorKind, Read, Write};
use std::net::TcpStream;
use std::thread;
use std::time::Duration;
use std::sync::mpsc::{self, TryRecvError};

const LOCAL: &str = "127.0.0.1:6000";
const MSG_SIZE: usize = 32;

fn main() {
    let mut client = TcpStream::connect(LOCAL).expect("Failed to connect");
    client.set_nonblocking(true).expect("Failed to initialize non-blocking");

    let (tx, rx) = mpsc::channel::<String>();

    thread::spawn(move || loop {
        let mut buffer = vec![0; MSG_SIZE];
        match client.read_exact(&mut buffer) {
            Ok(_) => {
                let message:Vec<u8> = buffer.into_iter().take_while(|&x| x != 0).collect();
                println!("Message received: {:?}", message);
            },
            Err(ref err) if err.kind() == ErrorKind::WouldBlock => (),
            Err(_) => {
                println!("Connection with server was severed");
                break;
            }
        }
        match rx.try_recv() {
            Ok(message) => {
                let mut buffer = message.clone().into_bytes();
                buffer.resize(MSG_SIZE, 0);
                client.write_all(&buffer).expect("Writing to socket failed");
                println!("Message sent: {:?}", message);
            },
            Err(TryRecvError::Empty) => (),
            Err(TryRecvError::Disconnected) => break
        }
        thread::sleep(Duration::from_millis(100));
    });

    println!("Write a message:");
    loop {
        let mut buffer = String::new();
        io::stdin().read_line(&mut buffer).expect("Reading from stdin failed");
        let message = buffer.trim().to_string();
        if message == ":quit" || tx.send(message).is_err() { break }
    }
    println!("Bye!");
}
