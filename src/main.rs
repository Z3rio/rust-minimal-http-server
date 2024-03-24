use std::{io::{Read, Write}, net::TcpListener};
use regex::Regex;
use itertools::Itertools;

const OK_RESPONSE: &str = "HTTP/1.1 200 OK";
const BAD_RESPONSE: &str = "HTTP/1.1 404 NOT FOUND";

fn main() {
    struct Route {
        name: Regex,
        method: String,
        handle_route: Box<dyn Fn(&str) -> String>
    }    

    fn index_handler(_raw_name: &str) -> String {
        return format!("{}\r\n\r\n", OK_RESPONSE.to_string());
    } 

    fn echo_handler(raw_name: &str) -> String {
        let resp_content = raw_name.replace("/echo/", "");
        return format!("{}\r\nContent-Type: text/plain\r\nContent-Length: {}\r\n\r\n{}", OK_RESPONSE, resp_content.len(), resp_content).to_string();
    }

    let routes: Vec<Route> = vec![
        Route {
            name: Regex::new("^\\/$").unwrap(),
            method: String::from("GET"),
            handle_route: Box::new(index_handler)
        },
        Route {
            name: Regex::new("^\\/echo\\/(.*)$").unwrap(),
            method: String::from("GET"),
            handle_route: Box::new(echo_handler)
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
                let req_lines = data.split("\r\n").collect_vec();
                let first_line_splits = req_lines[0].split(" ").collect_vec();
                let route_pos = routes.iter().position(|r| r.name.captures(first_line_splits[1]).is_some() && r.method == first_line_splits[0].to_string());

                match route_pos {
                    Some(route_pos) => {
                        stream.write((routes[route_pos].handle_route)(first_line_splits[1]).as_bytes()).unwrap();
                        stream.flush().unwrap();
                    }
                    None => {
                        stream.write(BAD_RESPONSE.as_bytes()).unwrap();
                        stream.flush().unwrap();
                    }
                }
            }
            Err(e) => {
                println!("error: {}", e);
            }
        }
    }
}
