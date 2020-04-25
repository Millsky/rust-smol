use std::env;
use std::convert::Infallible;
use futures::StreamExt;
use warp::{sse::ServerSentEvent, Filter};
use std::time::Duration;
use tokio::time::interval;

mod cassandra;

// create server-sent event
fn sse_counter(counter: u64) -> Result<impl ServerSentEvent, Infallible> {
    Ok((warp::sse::json(counter), warp::sse::event("tick")))
}

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

    let port : u16 = get_port();

    println!("Server is starting up on {}", port);
    let my_session : cassandra::CassandraSession = cassandra::authenticate();
    let health = warp::path!("health")
        .map(|| format!("All Good!"));

    let other_path = warp::path!("other_path")
        .map(|| format!("I'm Another Path"));

    let ticks = warp::path("ticks").and(warp::get()).map(|| {
        let mut counter: u64 = 0;
        let event_stream = interval(Duration::from_millis(1)).map(move |_| {
            counter += 1;
            sse_counter(counter)
        });
        warp::sse::reply(event_stream)
    });

    println!("Server is up and running on port {}", port);

    let routes = health
        .or(other_path)
        .or(ticks);

    // Start the Server!
    warp::serve(routes)
        .run(([0, 0, 0, 0], port))
        .await;
}
