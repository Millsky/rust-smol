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
    let health = warp::path!("health")
        .map(|| format!("All Gucci"));

    let other_path = warp::path!("other_path")
        .map(|| format!("I'm Another Path"));

    let port : u16 = get_port();

    println!("Server is up and running on port {}", port);

    let routes = health
        .or(other_path);

    warp::serve(routes)
        .run(([0, 0, 0, 0], port))
        .await;
}
