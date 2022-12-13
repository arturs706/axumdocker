use axum::{
    extract::State,
    Json,
    response::IntoResponse,
    http::{StatusCode, HeaderMap},  
};
use serde::{Serialize, Deserialize};
use sqlx::{self, FromRow};
use uuid::Uuid;
use serde_json::json;
use crate::AppState;
use core::fmt;
use std::borrow::Cow;
use tower_cookies::{Cookie, Cookies};
use jsonwebtoken::{Header, Algorithm, EncodingKey};
use chrono::{Utc, Duration};
use argon2::{password_hash::{rand_core::OsRng, SaltString},Argon2, PasswordVerifier};
use argon2::PasswordHash;
use argon2::PasswordHasher;



//User model for get all users
#[derive(Serialize, FromRow, Debug)]
struct User {
    usid : Uuid,
    fullname: String,
    username: String,
    dob: String,
    gender: String,
    mob_phone: String,
    email: String,
    created_at: chrono::DateTime<chrono::Utc>,
    address: String,
    city: String,
    postcode: String
}

//User model for register
#[derive(Serialize, Deserialize, Debug)]
pub struct UserReg {
    fullname: String,
    username: String,
    dob: String,
    gender: String,
    mob_phone: String,
    email: String,
    passwd: String,
    address: String,
    city: String,
    postcode: String
}

#[derive(Deserialize, FromRow, Debug)]
pub struct UserLoginUuid{
    usid: Uuid,
    passwd: String,
}

pub enum Role {
    User,
    Admin,
}

impl Role { pub fn _from_str(role: &str) -> Role {
    match role { 
        "Admin" =>  
                Role::Admin, 
        _ =>    Role::User, 
    }}}

impl fmt::Display for Role { fn fmt(&self, f: &mut fmt::Formatter<'_>) ->
    fmt::Result { 
    match self { 
    Role::User => write!(f, "User"), 
    Role::Admin => write!(f, "Admin"),
}}}
#[derive(Debug, Serialize, Deserialize)]

pub struct ClaimsAccessToken { 
    pub sub: Uuid,
    pub exp: i64, 
    pub iat: i64, 
    pub role: String,
    }

impl ClaimsAccessToken { 
    pub fn new (id: Uuid, role: Role ) -> Self { 
    let iat = Utc::now();
    let exp = iat + Duration::hours(24);
    Self {
        sub: id,
        iat: iat.timestamp(),
        exp: exp.timestamp(),
        role: role.to_string(),
}}}
#[derive(Debug, Serialize, Deserialize)]
pub struct ClaimsRefreshToken { 
    pub sub: Uuid,
    pub exp: i64, 
    pub iat: i64, 
    pub role: String,
    }


impl ClaimsRefreshToken { 
    pub fn new (id: Uuid, role: Role ) -> Self { 
    let iat = Utc::now();
    let exp = iat + Duration::hours(72);
    Self {
        sub: id,
        iat: iat.timestamp(),
        exp: exp.timestamp(),
        role: role.to_string(),
}}}

#[derive(Deserialize, FromRow, Debug)]
pub struct UserLogin{
    email: String,
    passwd: String,
}


//User registration route

//get all users

pub async fn fetchusershandler(State(state): State<AppState>) -> impl IntoResponse {
    let response = sqlx::query_as::<_, User>("SELECT users.usid, users.fullname, users.username, users.dob, users.gender, users.mob_phone, users.email, users.created_at, useraddr.address, useraddr.city, useraddr.postcode
    FROM users
    INNER JOIN useraddr ON users.usid = useraddr.userid")
    .fetch_all(&state.database.db)
    .await;
    match response {
        Ok(users) => (StatusCode::OK , Json(json!({
            "users": users
        }))),
        Err(e) => (StatusCode::INTERNAL_SERVER_ERROR, Json(json!({
            "status": "error",
            "message": "Something went wrong",
            "error": e.to_string(),
        }))),
    }
    
}

//user reg route
pub async fn regroute(State(state): State<AppState>, req: Json<UserReg>) -> impl IntoResponse {
    let usid = sqlx::types::Uuid::from_u128(uuid::Uuid::new_v4().as_u128()); 
    let addrid = sqlx::types::Uuid::from_u128(uuid::Uuid::new_v4().as_u128()); 
        let argon2 = Argon2::default();
        let salt = SaltString::generate(&mut OsRng);
        let password_hash = argon2.hash_password(req.passwd.as_bytes(), &salt).unwrap().to_string();
        let mut tx = state.database.db.begin().await.unwrap();
        let _cow = Cow::Borrowed("23505");
        let response = sqlx::query(
            "INSERT INTO users (usid, fullname, username, dob, gender, mob_phone, email, passwd, created_at) VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9)")
            .bind(usid)
            .bind(&req.fullname)
            .bind(&req.username)
            .bind(&req.dob)
            .bind(&req.gender)
            .bind(&req.mob_phone)
            .bind(&req.email)
            .bind(password_hash)
            .bind(chrono::Utc::now())
            .execute(&mut tx)
            .await;
            match response {
                Ok( _ ) => {
                    let response = sqlx::query(
                        "INSERT INTO useraddr (addrid, userid, address, city, postcode) VALUES ($1, $2, $3, $4, $5)")
                        .bind(addrid)
                        .bind(usid)
                        .bind(&req.address)
                        .bind(&req.city)
                        .bind(&req.postcode)
                        .execute(&mut tx)
                        .await;

                        
                        match response {
                            Ok(_) => {
                                tx.commit().await.unwrap();
                                (StatusCode::OK, Json(json!({
                                    "status": "success",
                                    "message": "User registered successfully"
                                })))
                            },
                            Err(e) => match e {
                                
                                sqlx::Error::Database(e) => {
                                    tx.rollback().await.unwrap();
                                    match e.code() {
                                        Some(_cow) => (StatusCode::BAD_REQUEST, Json(json!({
                                            "status": "error",
                                            "message": "User already exists",
                                        }))),
                                        _ => (StatusCode::INTERNAL_SERVER_ERROR, Json(json!({
                                            "status": "error",
                                            "message": "Something went wrong"
                                        }))),
                                    }
                                }
                                _ => (StatusCode::INTERNAL_SERVER_ERROR, Json(json!({
                                    "status": "error",
                                    "message": "Something went wrong"
                                }))),
                            },
                        
                        }

                },
                Err(e) => match e {
                                
                    sqlx::Error::Database(e) => {
                        tx.rollback().await.unwrap();

                        match e.code() {
                            Some(_cow) => (StatusCode::BAD_REQUEST, Json(json!({
                                "status": "error",
                                "message": "User already exists",
                            }))),
                            _ => (StatusCode::INTERNAL_SERVER_ERROR, Json(json!({
                                "status": "error",
                                "message": "Something went wrong"
                            }))),
                        }
                    }
                    _ => (StatusCode::INTERNAL_SERVER_ERROR, Json(json!({
                        "status": "error",
                        "message": "Something went wrong"
                    }))),
                },

        }

    }

    pub async fn loginuser(State(state): State<AppState>, cookies: Cookies, req: Json<UserLogin>,) -> impl IntoResponse {
        let mut headers = HeaderMap::new();
    
        if req.passwd.is_empty() || req.email.is_empty(){
            return (StatusCode::BAD_REQUEST,headers, Json(json!({
                "status": "error",
                "message": "Email or password cannot be empty"
            })));
        }
        let _cow = Cow::Borrowed("23505");
        let response =  sqlx::query_as::<_, UserLoginUuid>("SELECT * FROM users where email = $1", )
        .bind(&req.email)
        .fetch_one(&state.database.db)
        .await;
        match response {
            Ok(user) => {    
                let parsed_hash = PasswordHash::new(&user.passwd).unwrap();
                let is_pass_valid = Argon2::default().verify_password(req.passwd.as_bytes(), &parsed_hash).is_ok();
                if is_pass_valid {
                
                    let role_access = if req.email == "aradionovs@yahoo.com" { Role::Admin } else { Role::User };
                    let role_refresh = if req.email == "aradionovs@yahoo.com" { Role::Admin } else { Role::User };
                    let access_secret = &state.accesstoken.accesstoken.as_bytes();
                    let access_token = jsonwebtoken::encode(&Header::new(Algorithm::HS256), &ClaimsAccessToken::new(user.usid, role_access),&EncodingKey::from_secret(access_secret)).unwrap();
                    let refresh_secret = &state.refreshtoken.refreshtoken.as_bytes();
                    let refresh_stoken = jsonwebtoken::encode(&Header::new(Algorithm::HS256), &ClaimsRefreshToken::new(user.usid, role_refresh),&EncodingKey::from_secret(refresh_secret.as_ref())).unwrap();
                    // let bearertoken = format!("Bearer {}", access_token);
                    cookies.add(Cookie::build("Refresh Token", refresh_stoken.to_string())
                    .domain("axumtoyserver.shuttleapp.rs")
                    .path("/api/v1/users/login")
                    .secure(true)
                    .http_only(true)
                    .finish());
                    
                    headers.insert("Authorization", access_token.parse().unwrap());
                    (StatusCode::OK, headers, Json(json!({
                        "status": "success",
                        "message": "User logged in successfully",
                        "access_token": access_token.to_string(),
                        "refresh_token": refresh_stoken.to_string(),
                    })))
                } else {
                    (StatusCode::BAD_REQUEST, headers, Json(json!({
                        "status": "error",
                        "message": "Invalid email or password",
                    })))
                }
            }
            Err(e) => match e {
                sqlx::Error::Database(e) => {
                    match e.code() {
                        Some(_cow) => (StatusCode::BAD_REQUEST, headers, Json(json!({
                            "status": "error",
                            "message": "User already exists",
                        }))),
                        _ => (StatusCode::INTERNAL_SERVER_ERROR,headers, Json(json!({
                            "status": "error",
                            "message": "Something went wrong"
                        }))),
                    }
                }
                _ => (StatusCode::INTERNAL_SERVER_ERROR,headers, Json(json!({
                    "status": "error",
                    "message": "Something went wrong"
                }))),
            },
        }
        
    }
    