use axum::{Server, Router, routing::get, response::{Html, IntoResponse}, http::Response};
use tokio::{fs, net::TcpListener, io::{AsyncReadExt, AsyncWriteExt}};
use sha1::{Sha1, Digest};
use base64::{Engine as _, engine::general_purpose};


#[tokio::main]
async fn main() {
    let ix = 256;
    println!("{}", ix >> 8);
    println!("{}", ix & 0xff);

    let router  = Router::new()
        .route("/", get(get_root))
        .route("/index.js", get(indexjs_get));

    let server = Server::bind(&"0.0.0.0:7878".parse().unwrap()).serve(router.into_make_service());
    let addr = server.local_addr();
    println!("Listening on http://{}", addr);
    
    tokio::spawn(async move {
        create_web_socket().await;
    });

    server.await.unwrap();

} 

async fn get_root() -> impl IntoResponse {
    let html_page = fs::read_to_string("src/index.html").await.unwrap();

    Html(html_page)
}

async fn indexjs_get() -> impl IntoResponse {
    let js_page = fs::read_to_string("src/index.js").await.unwrap();

    Response::builder()
        .header("content-type", "text/javascript")
        .body(js_page)
        .unwrap()

}


// create a tcp listner on port 8330 for the websocket

async fn create_web_socket() {
    let listener = TcpListener::bind("0.0.0.0:8330").await.unwrap();
    println!("Listening for webSocket on: {}", listener.local_addr().unwrap());

    loop {
        let (stream, _) = listener.accept().await.unwrap();

        tokio::spawn(async move {
            let (mut reader, mut writer) = tokio::io::split(stream);
            setup_web_socket(&mut reader, &mut writer).await;


            writer.write_all(&generate_web_socket_message("message from server".to_string())).await.unwrap();
        });
    }
    
}

//function to parse the http request and return the websocket key
fn get_websocket_key_from_http(buf: &[u8]) -> String {
    let mut key = String::new();
    let mut lines = buf.split(|&b| b == b'\n');
    while let Some(line) = lines.next() {
        if line.starts_with(b"Sec-WebSocket-Key: ") {
            key = String::from_utf8(line[19..].to_vec()).unwrap();
            break;
        }
    }
    key.pop();
    key
}

// method for setting up the websocket

async fn setup_web_socket(reader: &mut tokio::io::ReadHalf<tokio::net::TcpStream>, writer: &mut tokio::io::WriteHalf<tokio::net::TcpStream>) {
    // setting up the websocket
    let mut buf = [0; 1024];
    let n = reader.read(&mut buf).await.unwrap();
    if n == 0 {
        return;
    }
    let key = get_websocket_key_from_http(&buf[0..n]);
    
    let new_key = key + "258EAFA5-E914-47DA-95CA-C5AB0DC85B11";
    let generated_key = general_purpose::STANDARD_NO_PAD.encode(&Sha1::digest(new_key));

    let response = format!("HTTP/1.1 101 Switching Protocols\r\nUpgrade: websocket\r\nConnection: Upgrade\r\nSec-WebSocket-Accept:{}=\r\nSec-WebSocket-Protocol: chat\r\n\r\n", generated_key);

    println!("Establishing connection with key {}", generated_key);

    writer.write_all(response.as_bytes()).await.unwrap();
    writer.flush().await.unwrap();
}


// metode for generating the websocket message from the server
fn generate_web_socket_message(message: String) -> Vec<u8> {
    let mut data = Vec::new();
    data.push(129); // 129 = 81 hex witch means binary 10000001 witch means fin = 1 and opcode = 1 this is for text
    let len = message.len();
    if len < 126 {
        data.push(len as u8);   // just adds lenght if lenght is less than 126
    } else if len < 65536 {
        data.push(126); // when the lenght is longer then 126 it adds 126 and then the lenght in 2 bytes
        data.push((len >> 8) as u8); //get the first 8 bits of the lenght
        data.push((len & 0xff) as u8); // get the last 8 bits of the lenght
    } else {
        data.push(127); // same as previus but with 8 bytes
        data.push((len >> 56) as u8);
        data.push((len >> 48) as u8);
        data.push((len >> 40) as u8);
        data.push((len >> 32) as u8);
        data.push((len >> 24) as u8);
        data.push((len >> 16) as u8);
        data.push((len >> 8) as u8);
        data.push((len & 0xff) as u8);
    }
    data.extend(message.as_bytes());
    data
}