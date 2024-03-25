use std::{env, fs::read_to_string, io::{Read, Write}, net::{TcpListener, TcpStream}, thread};
use regex::Regex;
use itertools::Itertools;
use once_cell::sync::Lazy;

const OK_RESPONSE: &str = "HTTP/1.1 200 OK";
const BAD_RESPONSE: &str = "HTTP/1.1 404 NOT FOUND";
const ARGS: Lazy<Vec<String>> = Lazy::new(|| {
    let args: Vec<String> = env::args().collect();
    return args;
});

fn index_handler(_raw_name: &str, _headers: Vec<&str>, _body: &str) -> String {
    return format!("{}\r\n\r\n", OK_RESPONSE.to_string());
} 

fn echo_handler(raw_name: &str, _headers: Vec<&str>, _body: &str) -> String {
    let resp_content = raw_name.replace("/echo/", "");
    return format!("{}\r\nContent-Type: text/plain\r\nContent-Length: {}\r\n\r\n{}", OK_RESPONSE, resp_content.len(), resp_content).to_string();
}

fn user_agent_handler(_raw_name: &str, headers: Vec<&str>, _body: &str) -> String {
    let user_agent_header = headers.clone().into_iter().position(|h| h.to_lowercase().contains("user-agent"));

    match user_agent_header {
        Some(user_agent_header) => {
            let resp_content = &(headers[user_agent_header].to_string())["user-agent: ".len()..];
            return format!("{}\r\nContent-Type: text/plain\r\nContent-Length: {}\r\n\r\n{}", OK_RESPONSE.to_string(), resp_content.len(), resp_content);
        }
        None => {
            return format!("{}\r\n\r\n", BAD_RESPONSE).to_string();
        }
    }
}

fn bad_response_handler(_raw_name: &str, _headers: Vec<&str>, body: &str) -> String {
    return format!("{}\r\n\r\n", BAD_RESPONSE).to_string();
}

fn get_file_handler(raw_name: &str, headers: Vec<&str>, body: &str) -> String {
    let file_name = &raw_name["/files/".len()..];
    let full_path = format!("{}{}", ARGS.get(2).expect("Directory arg not defined"), file_name);
    let contents = read_to_string(full_path);

    match contents {
        Ok(contents) => {
            return format!("{}\r\nContent-Type: application/octet-stream\r\nContent-Length: {}\r\n\r\n{}", OK_RESPONSE, contents.len(), contents).to_string();
        }

        Err(_) => {
            return bad_response_handler(raw_name, headers, body);
        }
    }
}

fn post_file_handler(raw_name: &str, headers: Vec<&str>, body: &str) -> String {
    let file_name = &raw_name["/files/".len()..];
    let full_path = format!("{}{}", ARGS.get(2).expect("Directory arg not defined"), file_name);
    
    println!("full_path: {}", full_path);
    println!("body:      {}", body);

    // let contents = read_to_string(full_path);

    // match contents {
    //     Ok(contents) => {
    //         return format!("{}\r\nContent-Type: application/octet-stream\r\nContent-Length: {}\r\n\r\n{}", OK_RESPONSE, contents.len(), contents).to_string();
    //     }

    //     Err(_) => {
    //         return bad_response_handler(raw_name, headers);
    //     }
    // }

    return String::from("");
}

struct Route<'a> {
    name: &'a str,
    route: &'a str,
    method: &'a str
}

fn get_route_method(route: &str) -> Box<dyn Fn(&str, Vec<&str>, &str) -> String> {
    if route == "index" {
        return Box::new(index_handler)
    } else if route == "echo" {
        return Box::new(echo_handler)
    } else if route == "user_agent" {
        return Box::new(user_agent_handler)
    } else if route == "get_file" {
        return Box::new(get_file_handler)
    } else if route == "post_file" {
        return Box::new(post_file_handler)
    } else {
        return Box::new(bad_response_handler)
    }
}

const ROUTES: &[Route] = &[
    Route {
        name: "index",
        route: "^\\/$",
        method: "GET"
    },
    Route {
        name: "echo",
        route: "^\\/echo\\/(.*)$",
        method: "GET"
    },
    Route {
        name: "user_agent",
        route: "^\\/user-agent$",
        method: "GET"
    },
    Route {
        name: "get_file",
        route: "^\\/files\\/(.*)$",
        method: "GET"
    },
    Route {
        name: "post_file",
        route: "^\\/files\\/(.*)$",
        method: "POST"
    }
];

fn stream_handler(mut stream: TcpStream) {
    let mut buffer = [0; 512];
    stream.read(&mut buffer).unwrap();

    let data = String::from_utf8_lossy(&buffer[..]);

    let req_lines = data.split("\r\n").collect_vec();
    let header_lines = req_lines.clone()[1..req_lines.len() - 2].to_vec();
    let body = req_lines[req_lines.len() - 1];

    let first_line_splits = req_lines[0].split(" ").collect_vec();
    let route_pos = ROUTES.iter().position(|r| Regex::new(r.route).unwrap().captures(first_line_splits[1]).is_some() && r.method == first_line_splits[0].to_string());
    
    match route_pos {
        Some(route_pos) => {
            stream.write(get_route_method(ROUTES[route_pos].name)(first_line_splits[1], header_lines, body).as_bytes()).unwrap();
            stream.flush().unwrap();
        }
        None => {
            stream.write(bad_response_handler(first_line_splits[1], header_lines, body).as_bytes()).unwrap();
            stream.flush().unwrap();
        }
    }
}

fn main() {
    println!("Logs from your program will appear here!");

    let listener = TcpListener::bind("127.0.0.1:4221").unwrap();

    for stream in listener.incoming() {
        match stream {
            Ok(stream) => {
                thread::spawn(move || {
                    stream_handler(stream)
                });
            }
            Err(e) => {
                println!("error: {}", e);
            }
        }
    }
}
