//! JSON API example
//!
//! Demonstrates JSON request/response handling
//!
//! Run with: cargo run --example json_api

use half_core::{Router, Server, Request, Response};
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize)]
struct User {
    id: u64,
    name: String,
    email: String,
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let mut router = Router::new();

    // GET /users - List all users
    router.get("/users", |_req: Request| async {
        let users = vec![
            User {
                id: 1,
                name: "Alice".to_string(),
                email: "alice@example.com".to_string(),
            },
            User {
                id: 2,
                name: "Bob".to_string(),
                email: "bob@example.com".to_string(),
            },
        ];

        Response::json(&users)
            .unwrap_or_else(|_| Response::internal_error())
    });

    // GET /users/:id - Get user by ID
    router.get("/users/:id", |req: Request| async move {
        let id = req.param("id").unwrap_or("0");

        let user = User {
            id: id.parse().unwrap_or(0),
            name: format!("User {}", id),
            email: format!("user{}@example.com", id),
        };

        Response::json(&user)
            .unwrap_or_else(|_| Response::internal_error())
    });

    println!("🚀 JSON API Server running on http://127.0.0.1:3000");
    println!("\nEndpoints:");
    println!("  GET  http://127.0.0.1:3000/users");
    println!("  GET  http://127.0.0.1:3000/users/:id");

    Server::new(router)
        .bind(([127, 0, 0, 1], 3000))
        .run()
        .await?;

    Ok(())
}
