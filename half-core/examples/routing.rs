//! Advanced routing example
//!
//! Demonstrates path parameters, query parameters, and different HTTP methods
//!
//! Run with: cargo run --example routing

use half_core::{Router, Server, Request, Response};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let mut router = Router::new();

    // GET /
    router.get("/", |_req: Request| async {
        Response::text("Half Framework - Routing Examples")
    });

    // GET /hello/:name
    router.get("/hello/:name", |req: Request| async move {
        let name = req.param("name").unwrap_or("World");
        Response::text(format!("Hello, {}!", name))
    });

    // GET /users/:user_id/posts/:post_id
    router.get("/users/:user_id/posts/:post_id", |req: Request| async move {
        let user_id = req.param("user_id").unwrap_or("?");
        let post_id = req.param("post_id").unwrap_or("?");

        Response::json(&serde_json::json!({
            "user_id": user_id,
            "post_id": post_id,
            "message": format!("Fetching post {} for user {}", post_id, user_id)
        }))
        .unwrap_or_else(|_| Response::text("Error"))
    });

    // GET /search?q=query
    router.get("/search", |req: Request| async move {
        let query = req.query_param("q").unwrap_or("(no query)");
        let page = req.query_param("page").unwrap_or("1");

        Response::json(&serde_json::json!({
            "query": query,
            "page": page,
            "results": []
        }))
        .unwrap_or_else(|_| Response::text("Error"))
    });

    // POST /submit
    router.post("/submit", |req: Request| async move {
        let body = req.body_string().unwrap_or_else(|_| String::from("(empty)"));

        Response::json(&serde_json::json!({
            "message": "Data received",
            "body_length": body.len()
        }))
        .unwrap_or_else(|_| Response::text("Error"))
    });

    // PUT /update/:id
    router.put("/update/:id", |req: Request| async move {
        let id = req.param("id").unwrap_or("?");

        Response::json(&serde_json::json!({
            "message": format!("Updated resource {}", id),
            "id": id
        }))
        .unwrap_or_else(|_| Response::text("Error"))
    });

    // DELETE /delete/:id
    router.delete("/delete/:id", |req: Request| async move {
        let id = req.param("id").unwrap_or("?");

        Response::json(&serde_json::json!({
            "message": format!("Deleted resource {}", id),
            "id": id
        }))
        .unwrap_or_else(|_| Response::text("Error"))
    });

    println!("🚀 Routing Server running on http://127.0.0.1:3000");
    println!("\nEndpoints:");
    println!("  GET    http://127.0.0.1:3000/");
    println!("  GET    http://127.0.0.1:3000/hello/:name");
    println!("  GET    http://127.0.0.1:3000/users/:user_id/posts/:post_id");
    println!("  GET    http://127.0.0.1:3000/search?q=keyword");
    println!("  POST   http://127.0.0.1:3000/submit");
    println!("  PUT    http://127.0.0.1:3000/update/:id");
    println!("  DELETE http://127.0.0.1:3000/delete/:id");

    Server::new(router)
        .bind(([127, 0, 0, 1], 3000))
        .run()
        .await?;

    Ok(())
}
