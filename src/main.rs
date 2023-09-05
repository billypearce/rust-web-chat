#[tokio::main]
async fn main() {
    match webserver::run().await {
        Ok(_) => (),
        Err(e) => eprintln!("{e}"), // TODO actual error handling
    }
}
