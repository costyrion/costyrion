use axum::{
    error_handling::HandleErrorLayer,
    extract::{Path, Query, State},
    http::StatusCode,
    response::IntoResponse,
    routing::{delete, get, patch},
    Json, Router,
};
use serde::{Deserialize, Serialize};
use std::{
    collections::HashMap,
    sync::{Arc, RwLock},
    time::Duration,
};
use tower::{BoxError, ServiceBuilder};
use tower_http::trace::TraceLayer;
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt};
use uuid::Uuid;

#[tokio::main]
async fn main() {
    tracing_subscriber::registry()
        .with(
            tracing_subscriber::EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| "costyrion=debug,tower_http=debug".into()),
        )
        .with(tracing_subscriber::fmt::layer())
        .init();

    let db = Db::default();
    let app = app(db);
    let listener = tokio::net::TcpListener::bind("0.0.0.0:3000").await.unwrap();
    tracing::debug!("listening on {}", listener.local_addr().unwrap());
    axum::serve(listener, app).await.unwrap();
}

fn app(db: Db) -> Router {
    Router::new()
        .route("/resource", get(resource_index).post(resource_create))
        .route(
            "/resource/:id",
            patch(resource_update).delete(resource_delete),
        )
        // Add middleware to all routes
        .layer(
            ServiceBuilder::new()
                .layer(HandleErrorLayer::new(|error: BoxError| async move {
                    if error.is::<tower::timeout::error::Elapsed>() {
                        Ok(StatusCode::REQUEST_TIMEOUT)
                    } else {
                        Err((
                            StatusCode::INTERNAL_SERVER_ERROR,
                            format!("Unhandled internal error: {error}"),
                        ))
                    }
                }))
                .timeout(Duration::from_secs(10))
                .layer(TraceLayer::new_for_http())
                .into_inner(),
        )
        .with_state(db)
}

async fn resource_index(
    pagination: Option<Query<Pagination>>,
    State(db): State<Db>,
) -> impl IntoResponse {
    let resources = db.read().unwrap();
    let Query(pagination) = pagination.unwrap_or_default();
    let resources = resources
        .values()
        .skip(pagination.offset.unwrap_or(0))
        .take(pagination.limit.unwrap_or(usize::MAX))
        .cloned()
        .collect::<Vec<_>>();
    Json(resources)
}

async fn resource_create(
    State(db): State<Db>,
    Json(payload): Json<CreateResource>,
) -> impl IntoResponse {
    let resource = Resource {
        id: Uuid::new_v4(),
        name: payload.name,
        status: true,
    };

    db.write().unwrap().insert(resource.id, resource.clone());
}

async fn resource_update(
    Path(id): Path<Uuid>,
    State(db): State<Db>,
    Json(input): Json<UpdateResource>,
) -> Result<impl IntoResponse, StatusCode> {
    let mut resource = db
        .read()
        .unwrap()
        .get(&id)
        .cloned()
        .ok_or(StatusCode::NOT_FOUND)?;

    if let Some(name) = input.name {
        resource.name = name;
    }

    if let Some(status) = input.status {
        resource.status = status;
    }

    db.write().unwrap().insert(resource.id, resource.clone());

    Ok(Json(resource))
}

async fn resource_delete(
    Path(id): Path<Uuid>,
    State(db): State<Db>,
) -> impl IntoResponse {
    if db.write().unwrap().remove(&id).is_some() {
        StatusCode::NO_CONTENT
    } else {
        StatusCode::NOT_FOUND
    }
}

type Db = Arc<RwLock<HashMap<Uuid, Resource>>>;

#[derive(Debug, Serialize, Clone)]
struct Resource {
    id: Uuid,
    name: String,
    status: bool,
}

#[derive(Debug, Deserialize)]
struct CreateResource {
    name: String,
}

#[derive(Debug, Deserialize)]
struct UpdateResource {
    name: Option<String>,
    status: Option<bool>,
}

#[derive(Debug, Deserialize, Default)]
pub struct Pagination {
    pub offset: Option<usize>,
    pub limit: Option<usize>,
}
