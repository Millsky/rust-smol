use std::env;
use std::convert::Infallible;
use futures::StreamExt;
use warp::{sse::ServerSentEvent, Filter};
use std::time::Duration;
use tokio::time::interval;

extern crate cdrs;

// Cassandra Database Setup
use cdrs::authenticators::StaticPasswordAuthenticator;
use cdrs::cluster::session::{new as new_session};
use cdrs::cluster::{ClusterTcpConfig, NodeTcpConfigBuilder};
use cdrs::load_balancing::RoundRobin;
use cdrs::query::*;
use retry::{retry, delay::Fixed, OperationResult};

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

    // Authenticate to our Cassandra Cluster, as configured in the docker-compose file
    // TODO: Move this into it's own crate
    let authenticator = StaticPasswordAuthenticator::new("cassandra", "cassandra");
    // Connect to the Auth Seeder Node
    let node = NodeTcpConfigBuilder::new("cassandra-node1:9042", authenticator).build();
    let cluster_config = ClusterTcpConfig(vec![node]);
    // Attempt to connect 10 times, it's important to handle this from the service level
    // For debugging purposes Health-Checks can be seen by running `docker ps` on the compose stack
    let my_session = retry(Fixed::from_millis(30000).take(10), || {
        println!("Looping... Attempting to authenticate to Apache Cassandra Instance");
        return match new_session(&cluster_config, RoundRobin::new()) {
            Ok(val) => OperationResult::Ok(val),
            Err(e) => {
                println!("{}", e);
                return OperationResult::Retry("Not Up Yet");
            },
        };
    }).unwrap();

    println!("Cassandra Instance is up and running, attempting to query");

    let create_ks: &'static str = "CREATE KEYSPACE IF NOT EXISTS test_ks WITH REPLICATION = { \
                                   'class' : 'SimpleStrategy', 'replication_factor' : 1 };";

    my_session.query(create_ks);

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
