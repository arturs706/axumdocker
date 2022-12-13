use axum::{
    routing::get,
    Router,
};

#[tokio::main]
async fn main() {
    // build our application with a single route
    let app = Router::new()
    
    .route("/", get(|| async { "Hello, World!" }))
    .route("/again", get(|| async { "Hello, World!" }));
    axum::Server::bind(&"0.0.0.0:10000".parse().unwrap())
        .serve(app.into_make_service())
        .await
        .unwrap();
}
 