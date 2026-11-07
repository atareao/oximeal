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
    let meal_plan_detail = paths["/api/meal-plan/{id}"]
        .as_object()
        .expect("/api/meal-plan/{id} debe ser un objeto");
    assert!(
        meal_plan_detail.contains_key("patch"),
        "/api/meal-plan/{{id}} debe exponer patch"
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

/// El documento OpenAPI expone los endpoints y esquemas de etiquetas.
#[tokio::test]
async fn openapi_json_includes_tags_endpoints_and_schemas() {
    let state = common::test_state().await;

    let (status, body) = common::request(state, "GET", "/api/openapi.json", None).await;
    assert_eq!(status, StatusCode::OK);

    let paths = body["paths"].as_object().expect("objeto paths");
    let tags = paths["/api/tags"]
        .as_object()
        .expect("/api/tags debe ser un objeto");
    assert!(tags.contains_key("get"), "/api/tags debe exponer get");
    assert!(tags.contains_key("post"), "/api/tags debe exponer post");

    let tag_detail = paths["/api/tags/{id}"]
        .as_object()
        .expect("/api/tags/{id} debe ser un objeto");
    assert!(
        tag_detail.contains_key("delete"),
        "/api/tags/{{id}} debe exponer delete"
    );

    let schemas = body["components"]["schemas"]
        .as_object()
        .expect("objeto components.schemas");
    for name in ["Tag", "TagInput", "RecipeTag"] {
        assert!(
            schemas.contains_key(name),
            "falta el esquema {name} en components.schemas"
        );
    }

    // `Recipe` y `RecipeSummary` documentan el campo `tags`.
    for name in ["Recipe", "RecipeSummary"] {
        assert!(
            schemas[name]["properties"].get("tags").is_some(),
            "{name} debe documentar la propiedad tags"
        );
    }
}

/// El documento OpenAPI expone el volcado de receta a la lista y su esquema.
#[tokio::test]
async fn openapi_json_includes_from_recipe_endpoint_and_schema() {
    let state = common::test_state().await;

    let (status, body) = common::request(state, "GET", "/api/openapi.json", None).await;
    assert_eq!(status, StatusCode::OK);

    let paths = body["paths"].as_object().expect("objeto paths");
    let from_recipe = paths["/api/shopping-list/from-recipe"]
        .as_object()
        .expect("/api/shopping-list/from-recipe debe ser un objeto");
    let post = from_recipe
        .get("post")
        .expect("/api/shopping-list/from-recipe debe exponer post");
    let responses = post["responses"].as_object().expect("objeto responses");
    for code in ["200", "400", "404"] {
        assert!(
            responses.contains_key(code),
            "falta la respuesta {code} en from-recipe"
        );
    }

    let schemas = body["components"]["schemas"]
        .as_object()
        .expect("objeto components.schemas");
    assert!(
        schemas.contains_key("RecipeToShoppingListInput"),
        "falta el esquema RecipeToShoppingListInput en components.schemas"
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

/// El documento OpenAPI expone los endpoints y esquemas de ajustes del hogar.
#[tokio::test]
async fn openapi_json_includes_settings_endpoints_and_schemas() {
    let state = common::test_state().await;

    let (status, body) = common::request(state, "GET", "/api/openapi.json", None).await;
    assert_eq!(status, StatusCode::OK);

    let paths = body["paths"].as_object().expect("objeto paths");
    let settings = paths["/api/settings"]
        .as_object()
        .expect("/api/settings debe ser un objeto");
    assert!(
        settings.contains_key("get"),
        "/api/settings debe exponer get"
    );
    assert!(
        settings.contains_key("put"),
        "/api/settings debe exponer put"
    );

    let schemas = body["components"]["schemas"]
        .as_object()
        .expect("objeto components.schemas");
    for name in ["HouseholdSettings", "SettingsInput"] {
        assert!(
            schemas.contains_key(name),
            "falta el esquema {name} en components.schemas"
        );
    }
}
