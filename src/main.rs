use std::net::TcpListener;

use zero2prod_practice::run;

#[tokio::main]
async fn main() -> std::io::Result<()> {
    let listener = TcpListener::bind("0.0.0.0:8000")?;
    run(listener)?.await
}
