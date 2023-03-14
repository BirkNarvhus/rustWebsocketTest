use axum::{Server, Router, routing::get, response::{Html, IntoResponse}, http::Response};
use tokio::{fs, net::TcpListener, io::{AsyncReadExt, AsyncWriteExt}, sync::broadcast};
use sha1::{Sha1, Digest};
use base64::{Engine as _, engine::general_purpose};

#[tokio::main]
async fn main() {
    let router  = Router::new()
        .route("/", get(get_root))
        .route("/index.js", get(indexjs_get));

    let server = Server::bind(&"10.0.0.22:7878".parse().unwrap()).serve(router.into_make_service());
    let addr = server.local_addr();
    println!("Listening on http://{}", addr);
    

    // server to handel websocket connections
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
    let listener = TcpListener::bind("10.0.0.22:8330").await.unwrap();
    let addr = listener.local_addr().unwrap();
    println!("Listening for webSocket on: {}", addr);

    let (tx, _rx) = broadcast::channel(16);
    

    loop {
        let (stream, webs_addr) = listener.accept().await.unwrap();

        let tx = tx.clone();
        let mut rx = tx.subscribe();

        tokio::spawn(async move {

            let (mut reader, mut writer) = tokio::io::split(stream);
            setup_web_socket(&mut reader, &mut writer).await;
            println!("Connection established on: {}", webs_addr);

            let mut buff = [0; 1024];
            'inner : loop{
                tokio::select! {
                    result = reader.read(&mut buff) => {
                        let res = result.unwrap();

                        if res == 0 {
                            continue;
                        }

                        // checks if the opcode is closeing the connection
                        // and sends the same close connection message back to the client
                        if  is_close_code(&buff[0..res]) {
                            println!("Connection closed on : {}", webs_addr);
                            writer.write_all(&buff[0..res]).await.unwrap();
                            break 'inner;
                        }
                        

                        // unmask the message and prints it to the console
                        let message = unmask_message(&buff[0..res]);
                    
                        // sends the unmasked message to all the other clients
                        tx.send((message, webs_addr)).unwrap();
                    }
                    result = rx.recv() => {
                        let (message, rec_addr) = result.unwrap();
                        if rec_addr != webs_addr {
                            writer.write_all(&generate_web_socket_message(message)).await.unwrap();

                        }
                    }
                }
            }
            
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
    let mut buf = [0; 1024];
    let n = reader.read(&mut buf).await.unwrap();
    if n == 0 {
        return;
    }
    let key = get_websocket_key_from_http(&buf[0..n]);
    
    let new_key = key + "258EAFA5-E914-47DA-95CA-C5AB0DC85B11";
    let generated_key = general_purpose::STANDARD_NO_PAD.encode(&Sha1::digest(new_key));

    // the response to the http request for setting up websocket
    let response = format!("HTTP/1.1 101 Switching Protocols\r\nUpgrade: websocket\r\nConnection: Upgrade\r\nSec-WebSocket-Accept:{}=\r\nSec-WebSocket-Protocol: chat\r\n\r\n", generated_key);


    writer.write_all(response.as_bytes()).await.unwrap();
    writer.flush().await.unwrap();
}


// method for checking opcode of websocket message
fn is_close_code(message: &[u8]) -> bool {
    let code = message[0] & 0b00001111;
    code == 8
}


// metode for generating the websocket message from the server
fn generate_web_socket_message(message: Vec<u8>) -> Vec<u8> {
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
    data.extend(message);
    data
}

// method for unmasking the websocket message from client
fn unmask_message(message: &[u8]) -> Vec<u8> {
    let mut data = Vec::new();
    let mask = &message[2..6];
    let mut i = 0;
    for byte in &message[6..] {
        data.push(byte ^ mask[i % 4]);
        i += 1;
    }
    data
}