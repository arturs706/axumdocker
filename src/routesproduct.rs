use axum_macros::debug_handler;
use serde::{Deserialize, Serialize};
use sqlx::{self, FromRow};
use uuid::Uuid;
use crate::AppState;
use axum::{Json, extract::{Path, State}, response::IntoResponse, http::StatusCode};
use serde_json::json;

#[derive(Serialize, FromRow, Debug)]

struct Products {
    productid : Uuid,
    prodname: String,
    proddescr: String,
    prodsku: String,
    descr: String,
    availableqty: i64,
    price: String,
    imageone: String,
    imagetwo: String,
    imagethree: String,
    imagefour: String,
    created_at: chrono::DateTime<chrono::Utc>
}
#[derive(Serialize, Deserialize, FromRow, Debug)]

pub struct ProductUpdate {
    prodname: Option<String>,
    proddescr: Option<String>,
    prodsku: Option<String>,
    category: Option<Uuid>,
    availableqty: Option<i64>,
    price: Option<String>
}
#[derive(Serialize, Deserialize, FromRow, Debug)]

struct FavProducts {
    productid : Uuid,
    prodname: String,
    price: String,
    imagetwo: String
}


pub async fn fetchproductshandler(State(state): State<AppState>)-> impl IntoResponse {
    let response = sqlx::query_as::<_, Products>(
        "SELECT products.productid, products.prodname, products.proddescr, products.prodsku, products.availableqty, products.price, products.created_at, prodcategory.descr, productimages.imageone, productimages.imagetwo, productimages.imagethree, productimages.imagefour
        FROM products
        INNER JOIN prodcategory 
        ON products.category  = prodcategory.descr
        INNER JOIN productimages
        ON products.prodsku = productimages.prodskuid"
)
    .fetch_all(&state.database.db)
    .await;
    match response {
        Ok(products) => (StatusCode::OK , Json(json!({
            "products": products
        }))),
        Err(e) => (StatusCode::INTERNAL_SERVER_ERROR, Json(json!({
            "status": "error",
            "message": "Something went wrong",
            "error": e.to_string(),
        }))),
    }
    
}

#[debug_handler]
pub async fn fetchproducthandler (State(state): State<AppState>, Path(productid): Path<Uuid>) -> impl IntoResponse {
let response = sqlx::query_as::<_, Products>(
"SELECT 
products.productid, products.prodname, products.proddescr, products.prodsku, products.availableqty, products.price, products.created_at, prodcategory.descr, productimages.imageone, productimages.imagetwo, productimages.imagethree, productimages.imagefour
FROM products
INNER JOIN prodcategory 
ON products.category  = prodcategory.descr
INNER JOIN productimages
ON products.prodsku = productimages.prodskuid
where productid = $1"
)
    .bind(productid)
    .fetch_all(&state.database.db)
    .await;
    match response {
        Ok(product) => (StatusCode::OK , Json(json!({
            "product": product
        }))),
        Err(e) => (StatusCode::INTERNAL_SERVER_ERROR, Json(json!({
            "status": "error",
            "message": "Something went wrong",
            "error": e.to_string(),
        }))),
    }
}


#[debug_handler]
pub async fn deleteproducthandler (State(state): State<AppState>, Path(productid): Path<Uuid>) -> impl IntoResponse {
let response = sqlx::query_as::<_, Products>(
    "DELETE FROM products where productid = $1 ")
    .bind(productid)
    .fetch_optional(&state.database.db)
    .await;
    match response {
        Ok(_) => (StatusCode::OK , Json(json!({
            "product": "deleted"
        }))),
        Err(e) => (StatusCode::INTERNAL_SERVER_ERROR, Json(json!({
            "status": "error",
            "message": "Something went wrong",
            "error": e.to_string(),
        }))),
    }
}


#[debug_handler]
pub async fn updateproducthandler(State(state): State<AppState>, Path(productid): Path<Uuid>, Json(req): Json<ProductUpdate>) -> impl IntoResponse {
let response = sqlx::query_as::<_, ProductUpdate>(
    "
    UPDATE products 
    SET
    prodname = COALESCE(NULLIF($1, ''), prodname),
    proddescr = COALESCE(NULLIF($2, ''), proddescr),
    prodsku = COALESCE(NULLIF($3, ''), prodsku),
    availableqty = COALESCE(NULLIF($4, 0), availableqty),
    price = COALESCE(NULLIF($5, ''), price),
    category = (select descr from prodcategory where descr = 
    COALESCE(NULLIF($6, ''), category))
    WHERE productid = $7
")

    .bind(&req.prodname)
    .bind(&req.proddescr)
    .bind(&req.prodsku)
    .bind(&req.availableqty)
    .bind(&req.price)
    .bind(&req.category)
    .bind(productid)
    .fetch_optional(&state.database.db)
    .await;
    match response {
        Ok(_) => (StatusCode::OK , Json(json!({
            "updated": "success"
        }))),
        Err(e) => (StatusCode::INTERNAL_SERVER_ERROR, Json(json!({
            "status": "error",
            "message": "Something went wrong",
            "error": e.to_string(),
        }))),
    }
}




pub async fn addfavouriteitems(State(state): State<AppState>, Path((userid, productid)): Path<(Uuid, Uuid)>) -> impl IntoResponse {
    let favid = sqlx::types::Uuid::from_u128(uuid::Uuid::new_v4().as_u128()); 
    let response = sqlx::query(
        "
        INSERT INTO favourites(favid, userid, productid)
        VALUES ($1, $2, $3)
        ")
        .bind(favid)
        .bind(userid)
        .bind(productid)
        .fetch_optional(&state.database.db)
        .await;
        match response {
            Ok(_) => (StatusCode::OK , Json(json!({
                "favourite": "added"
            }))),
            Err(e) => (StatusCode::INTERNAL_SERVER_ERROR, Json(json!({
                "status": "error",
                "message": "Something went wrong",
                "error": e.to_string(),
            }))),
        }
    }

    pub async fn fetchfavouriteitems(State(state): State<AppState>, Path(usid): Path<Uuid>) -> impl IntoResponse {
        let response = sqlx::query_as::<_, FavProducts>(
            "SELECT products.productid, products.prodname, products.price, productimages.imagetwo
            FROM products
            INNER JOIN prodcategory 
            ON products.category  = prodcategory.descr
            INNER JOIN productimages
            ON products.prodsku = productimages.prodskuid
            INNER JOIN favourites
            ON products.productid = favourites.productid
            where favourites.userid = $1"

        )
            .bind(usid)
            .fetch_all(&state.database.db)
            .await;
            match response {
                Ok(product) => (StatusCode::OK , Json(json!({
                    "productlistres": product
                }))),
                Err(e) => (StatusCode::INTERNAL_SERVER_ERROR, Json(json!({
                    "status": "error",
                    "message": "Something went wrong",
                    "error": e.to_string(),
                }))),
            }
        }



        #[debug_handler]
        // path with userid and productid


pub async fn deletefavorite(State(state): State<AppState>, Path((userid, productid)): Path<(Uuid, Uuid)>) -> impl IntoResponse {
let response = sqlx::query(
    "delete from favourites
    where userid = $1
    AND productid  = $2
    ")
    .bind(userid)
    .bind(productid)
    .fetch_optional(&state.database.db)
    .await;
    match response {
        Ok(_) => (StatusCode::OK , Json(json!({
            "favourite": "deleted"
        }))),
        Err(e) => (StatusCode::INTERNAL_SERVER_ERROR, Json(json!({
            "status": "error",
            "message": "Something went wrong",
            "error": e.to_string(),
        }))),
    }
}

 


 