//! Simple Hello World example
//!
//! Run with: cargo run --example hello

use half_core::{Router, Server, Request, Response};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let mut router = Router::new();

    // Simple hello world route
    router.get("/", |_req: Request| async {
        Response::text("Hello, Half Framework!")
    });

    println!("🚀 Server running on http://127.0.0.1:3000");
    println!("Visit http://127.0.0.1:3000 to see the message");

    Server::new(router)
        .bind(([127, 0, 0, 1], 3000))
        .run()
        .await?;

    Ok(())
}
