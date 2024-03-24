use std::{io::{Read, Write}, net::TcpListener};

use itertools::Itertools;

const OK_RESPONSE: &str = "HTTP/1.1 200 OK\r\n\r\n";
const BAD_RESPONSE: &str = "HTTP/1.1 404 NOT FOUND\r\n\r\n";

fn main() {
    struct Route {
        name: String,
        method: String
    }    

    let routes: Vec<Route> = vec![
        Route {
            name: String::from("/"),
            method: String::from("GET")
        }
    ];

    println!("Logs from your program will appear here!");

    let listener = TcpListener::bind("127.0.0.1:4221").unwrap();

    for stream in listener.incoming() {
        match stream {
            Ok(mut stream) => {
                let mut buffer = [0; 512];
                stream.read(&mut buffer).unwrap();

                let data = String::from_utf8_lossy(&buffer[..]);
                let splits = data.split("\r\n").collect_vec();
                let splits2 = splits[0].split(" ").collect_vec();

                if routes.iter().any(|r| r.name == splits2[1].to_string() && r.method == splits2[0].to_string()) {
                    stream.write(OK_RESPONSE.as_bytes()).unwrap();
                    stream.flush().unwrap();
                } else {
                    stream.write(BAD_RESPONSE.as_bytes()).unwrap();
                    stream.flush().unwrap();
                }
            }
            Err(e) => {
                println!("error: {}", e);
            }
        }
    }
}
