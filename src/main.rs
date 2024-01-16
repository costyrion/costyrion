use std::env;

use axum::body::Body;
use axum::http::Response;
use axum::{
    extract::Path,
    http::StatusCode,
    routing::{delete, get, post},
    Json, Router,
};
use mongodb::{options::ClientOptions, Client};
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize)]
struct Resource {
    #[serde(skip_serializing_if = "Option::is_none")]
    id: Option<u64>,
    name: String,
}

#[tokio::main]
async fn main() {
    let uri = env::var("MONGODB_URL").expect("set environment var: MONGODB_URL");
    let mut options = ClientOptions::parse(&uri).await.unwrap();
    options.app_name = Some("Costyrion".to_string());
    let client = Client::with_options(options).unwrap(); // Unwrap the Result to get the Client instance
    let db = client.database("costyrion");

    let app = app();
    let listener = tokio::net::TcpListener::bind("0.0.0.0:3000").await.unwrap();
    axum::serve(listener, app).await.unwrap();
}

fn app() -> Router {
    Router::new()
        .route("/", get(handler))
        .route("/resource", post(create_resource))
        .route("/resource/:id", delete(delete_resource).get(get_resource))
}

async fn handler() -> String {
    "Costyrion!".to_owned()
}

async fn create_resource(Json(payload): Json<Resource>) -> (StatusCode, Json<Resource>) {
    let resource = Resource {
        id: Some(1337),
        name: payload.name,
    };

    (StatusCode::CREATED, Json(resource))
}

async fn delete_resource(Path(id): Path<u64>) -> Result<Response<Body>, StatusCode> {
    let message = format!("Resource with id: {} deleted successfully", id);
    Ok(Response::new(Body::from(message)))
}

async fn get_resource(Path(id): Path<u64>) -> Result<Json<Resource>, StatusCode> {
    let resource = Resource {
        id: Some(id),
        name: "Costyrion".to_string(),
    };

    Ok(Json(resource))
}

#[cfg(test)]
mod tests {
    use super::*;
    use axum::{
        body::Body,
        http::{Request, StatusCode},
    };
    use http_body_util::BodyExt;
    use tower::ServiceExt; // for `collect`

    #[tokio::test]
    async fn test_app() {
        let app = app();

        let response = app
            .oneshot(Request::builder().uri("/").body(Body::empty()).unwrap())
            .await
            .unwrap();

        assert_eq!(response.status(), StatusCode::OK);

        let body = response.into_body().collect().await.unwrap().to_bytes();
        assert_eq!(&body[..], b"Costyrion!");
    }

    #[tokio::test]
    async fn test_create_resource() {
        let app = app();
        let resource = Resource {
            id: None,
            name: "Test Resource".to_owned(),
        };
        let payload = serde_json::to_vec(&resource).unwrap();

        let request = Request::builder()
            .method("POST")
            .uri("/resource")
            .header("content-type", "application/json")
            .body(Body::from(payload))
            .unwrap();

        let response = app.oneshot(request).await.unwrap();

        assert_eq!(response.status(), StatusCode::CREATED);

        let body = response.into_body().collect().await.unwrap().to_bytes();
        let returned_resource: Resource = serde_json::from_slice(&body).unwrap();

        assert_eq!(returned_resource.id, Some(1337));
        assert_eq!(returned_resource.name, resource.name);
    }

    #[tokio::test]
    async fn test_delete_resource() {
        let app = app();
        let id = 1337;

        let request = Request::builder()
            .method("DELETE")
            .uri(format!("/resource/{}", id))
            .body(Body::empty())
            .unwrap();

        let response = app.oneshot(request).await.unwrap();

        assert_eq!(response.status(), StatusCode::OK);

        let body = response.into_body().collect().await.unwrap().to_bytes();
        let message = std::str::from_utf8(&body).unwrap();
        let expected_message = format!("Resource with id: {} deleted successfully", id);

        assert_eq!(message, expected_message);
    }

    #[tokio::test]
    async fn test_get_resource() {
        let app = app();
        let id = 1337;

        let request = Request::builder()
            .method("GET")
            .uri(format!("/resource/{}", id))
            .body(Body::empty())
            .unwrap();

        let response = app.oneshot(request).await.unwrap();

        assert_eq!(response.status(), StatusCode::OK);

        let body = response.into_body().collect().await.unwrap().to_bytes();
        let returned_resource: Resource = serde_json::from_slice(&body).unwrap();

        assert_eq!(returned_resource.id, Some(id));
        assert_eq!(returned_resource.name, "Costyrion");
    }
}
