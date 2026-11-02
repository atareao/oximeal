//! Tests de integración del health check.

mod common;

use axum::body::Body;
use axum::http::{Request, StatusCode};
use http_body_util::BodyExt;
use oximeal_backend::app;
use tower::ServiceExt;

#[tokio::test]
async fn health_returns_200_and_ok_status() {
    let state = common::test_state().await;

    let response = app(state)
        .oneshot(
            Request::builder()
                .uri("/api/health")
                .body(Body::empty())
                .expect("construir request"),
        )
        .await
        .expect("ejecutar request");

    assert_eq!(response.status(), StatusCode::OK);

    let body = response
        .into_body()
        .collect()
        .await
        .expect("leer body")
        .to_bytes();
    let json: serde_json::Value = serde_json::from_slice(&body).expect("parsear JSON");

    assert_eq!(json["status"], "ok");
    assert_eq!(json["db"], "ok");
}
