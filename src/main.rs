use axum::{Server, Router, routing::get};


#[tokio::main]
async fn main() {
    let router  = Router::new()
        .route("/", get(get_root));

    let server = Server::bind(&"0.0.0.0:7878".parse().unwrap()).serve(router.into_make_service());
    let addr = server.local_addr();
    println!("Listening on http://{}", addr);
    
    server.await.unwrap();
    
} 

async fn get_root() -> &'static str {
    "Hello, World!"
}