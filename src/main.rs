use std::env;
use warp::Filter;

fn get_port() -> u16 {
    match env::var("PORT") {
        Ok(val) => val.parse().unwrap(),
        Err(e) => {
            println!("{}", e);
            0
        },
    }
}

#[tokio::main]
async fn main() {
    let hello = warp::path!("hello" / String)
        .map(|name| format!("Hello, {}!", name));
    let port : u16 = get_port();
    println!("Server is up and running on port {}", port);
    warp::serve(hello)
        .run(([0, 0, 0, 0], port))
        .await;
}
