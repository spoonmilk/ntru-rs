use ntru::convolution_polynomial::ConvPoly;
use ntru::ntru_key::NtruKeyPair;
use std::env;
use std::io::{self, Read, Write};
use std::net::{TcpListener, TcpStream};
use std::thread;

fn run_server() {
    let keypair = NtruKeyPair::new();
    let listener = TcpListener::bind("127.0.0.1:7878").expect("Failed to bind to port 7878");

    println!("Server listening on 127.0.0.1:7878");

    for stream in listener.incoming() {
        let keypair = keypair.clone();
        match stream {
            Ok(stream) => {
                thread::spawn(move || {
                    handle_client(stream, keypair);
                });
            }
            Err(e) => eprintln!("Connection failed: {}", e),
        }
    }
}

fn handle_client(mut stream: TcpStream, keypair: NtruKeyPair) {
    let mut buffer = vec![0; 4096];
    stream
        .read(&mut buffer)
        .expect("Failed to read from client");

    let enc_poly = ConvPoly::from_be_bytes(&buffer);
    let decrypted = keypair.private.decrypt_to_bytes(enc_poly);
    let message = String::from_utf8(decrypted).expect("Invalid UTF-8");

    println!("Received and decrypted message: {}", message);
}

fn run_client() {
    let keypair = NtruKeyPair::new();
    let public_key = keypair.public;

    println!("Connected to server. Enter messages to send (max 100 bytes). Type 'exit' to quit.");

    loop {
        print!("Enter message: ");
        io::stdout().flush().expect("Failed to flush stdout");

        let mut input = String::new();
        io::stdin()
            .read_line(&mut input)
            .expect("Failed to read input");

        let message = input.trim();
        if message == "exit" {
            println!("Exiting client.");
            break;
        }

        if message.len() > 100 {
            println!("Message too long. Please keep it under 100 bytes.");
            continue;
        }

        let message_bytes = message.as_bytes().to_vec();
        let enc_poly = public_key.encrypt_bytes(message_bytes);
        let enc_bytes = enc_poly.to_be_bytes();

        let mut stream = TcpStream::connect("127.0.0.1:7878").expect("Failed to connect to server");
        stream
            .write_all(&enc_bytes)
            .expect("Failed to send message");

        println!("Sent encrypted message.");
    }
}

fn main() {
    let args: Vec<String> = env::args().collect();

    if args.len() < 2 {
        eprintln!("Usage: {} [server|client]", args[0]);
        std::process::exit(1);
    }

    match args[1].as_str() {
        "server" => run_server(),
        "client" => run_client(),
        _ => {
            eprintln!("Invalid argument. Use 'server' or 'client'.");
            std::process::exit(1);
        }
    }
}