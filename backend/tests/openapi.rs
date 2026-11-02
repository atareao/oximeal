//! Tests de integración de la documentación OpenAPI y Swagger UI.

mod common;

use axum::http::StatusCode;

#[tokio::test]
async fn openapi_json_is_served_with_expected_paths() {
    let state = common::test_state().await;

    let (status, body) = common::request(state, "GET", "/api/openapi.json", None).await;
    assert_eq!(status, StatusCode::OK);

    assert!(
        body.get("openapi").is_some(),
        "el documento debe declarar la versión OpenAPI"
    );
    assert_eq!(
        body["info"]["version"],
        env!("CARGO_PKG_VERSION"),
        "la versión del documento debe derivarse de la versión del paquete"
    );
    let paths = body["paths"].as_object().expect("objeto paths");
    assert!(
        paths.contains_key("/api/recipes"),
        "falta /api/recipes en paths"
    );
    let recipe_detail = paths["/api/recipes/{id}"]
        .as_object()
        .expect("/api/recipes/{id} debe ser un objeto");
    for method in ["get", "put", "delete"] {
        assert!(
            recipe_detail.contains_key(method),
            "/api/recipes/{{id}} debe exponer {method}"
        );
    }
    assert!(
        paths.contains_key("/api/meal-plan"),
        "falta /api/meal-plan en paths"
    );
    assert!(
        paths.contains_key("/api/shopping-list"),
        "falta /api/shopping-list en paths"
    );
    assert!(
        paths.contains_key("/api/health"),
        "falta /api/health en paths"
    );
}

#[tokio::test]
async fn swagger_ui_is_served() {
    let state = common::test_state().await;

    let (status, _) = common::request(state, "GET", "/api/docs", None).await;
    assert_eq!(status, StatusCode::OK);
}

#[tokio::test]
async fn swagger_ui_assets_resolve_under_docs_path() {
    let state = common::test_state().await;

    let (status, _) = common::request(state, "GET", "/api/docs/swagger-ui.css", None).await;
    assert_eq!(
        status,
        StatusCode::OK,
        "los recursos deben resolverse bajo /api/docs/"
    );
}
