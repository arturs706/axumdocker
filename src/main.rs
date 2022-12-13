use axum::{routing::{get, post},Router};
use dotenv::dotenv;
use sqlx::{postgres::PgPoolOptions, Pool, Postgres};
use tower_http::cors::CorsLayer;
use http::Method;
mod routesuser;
use tower_cookies::CookieManagerLayer;

#[derive(Clone)]
pub struct AppState {
    pub database: Database,
    pub accesstoken: AccessToken,
    pub refreshtoken: RefreshToken,
    pub passrecovertoken: PasswordRecoveryToken,
    pub stripetoken: StripeToken,
    pub stripepubtoken: StripePublicToken
}

#[derive(Clone)]
pub struct Database {
    pub db: Pool<Postgres>,
}
#[derive(Clone)]
pub struct AccessToken {
    pub accesstoken: String
}
#[derive(Clone)]
pub struct RefreshToken {
    pub refreshtoken: String
}
#[derive(Clone)]
pub struct PasswordRecoveryToken {
    pub passrecovertoken: String
}
#[derive(Clone)]
pub struct StripeToken {
    pub stripetoken: String
}
#[derive(Clone)]
pub struct StripePublicToken {
    pub stripepubtoken: String
}



#[tokio::main]
async fn main() {
    dotenv().ok();
    let database_url: String = std::env::var("DATABASE_URL").expect("DATABASE_URL must be set");
    let access_token_secret: String = std::env::var("ACCESS_TOKEN_SECRET").expect("ACCESS_TOKEN_SECRET must be set");
    let refresh_token_secret: String = std::env::var("REFRESH_TOKEN_SECRET").expect("REFRESH_TOKEN_SECRET must be set");
    let stripe_token_secret: String = std::env::var("STRIPE_SECRET_KEY").expect("STRIPE_SECRET_KEY must be set");
    let stripe_public_secret: String = std::env::var("STRIPE_PUBLISH_KEY").expect("STRIPE_PUBLISH_KEY must be set");
    let reset_passwprd_secret: String = std::env::var("RESET_PASSWORD_SECRET").expect("RESET_PASSWORD_SECRET must be set");
    let cors = CorsLayer::new()
    .allow_methods(vec![Method::GET, Method::POST, Method::OPTIONS])
    .allow_credentials(true);
    let pool = PgPoolOptions::new()
    .max_connections(5)
    .connect(&database_url)
    .await
    .expect("Failed to create pool");
    let state = AppState { 
        database: Database { db: pool },
        accesstoken: AccessToken { accesstoken: access_token_secret },
        refreshtoken: RefreshToken { refreshtoken: refresh_token_secret },
        passrecovertoken: PasswordRecoveryToken { passrecovertoken: reset_passwprd_secret },
        stripetoken: StripeToken { stripetoken: stripe_token_secret },
        stripepubtoken: StripePublicToken { stripepubtoken: stripe_public_secret }
    };
    let app = Router::new()
    .route("/api/v1/users", get(routesuser::fetchusershandler))
    .route("/api/v1/users/register", post(routesuser::regroute))
    .route("/api/v1/users/login", post(routesuser::loginuser))
    .layer(cors)
    .layer(CookieManagerLayer::new())
    .with_state(state);


    axum::Server::bind(&"0.0.0.0:10000".parse().unwrap())
        .serve(app.into_make_service())
        .await
        .unwrap();
}
 