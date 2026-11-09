//! Documento OpenAPI y Swagger UI.

use std::sync::{Arc, OnceLock};

use axum::Json;
use axum::Router;
use axum::body::Body;
use axum::extract::Path;
use axum::http::{HeaderMap, HeaderValue, StatusCode, header};
use axum::response::{IntoResponse, Response};
use axum::routing::get;
use utoipa::OpenApi;
use utoipa_swagger_ui::{Config, Url, serve};

use crate::state::AppState;

use super::{agent, health, meal_plan, recipes, settings, shopping_list, tags};

/// Ruta pública del documento OpenAPI.
const OPENAPI_PATH: &str = "/api/openapi.json";
/// Ruta pública de la Swagger UI.
const DOCS_PATH: &str = "/api/docs";

/// Documento OpenAPI de la API de OxiMeal.
#[derive(OpenApi)]
#[openapi(
    info(
        title = "OxiMeal API",
        version = env!("CARGO_PKG_VERSION"),
        description = "API del gestor self-hosted de menús, recetas y listas de la compra."
    ),
    paths(
        health::health,
        recipes::list_recipes,
        recipes::create_recipe,
        recipes::import_recipe,
        recipes::get_recipe,
        recipes::update_recipe,
        recipes::autotag_recipe,
        recipes::delete_recipe,
        tags::list_tags,
        tags::create_tag,
        tags::delete_tag,
        settings::get_settings,
        settings::update_settings,
        meal_plan::list_meal_plan,
        meal_plan::create_meal_plan,
        meal_plan::patch_meal_plan,
        meal_plan::delete_meal_plan,
        shopping_list::list_shopping_list,
        shopping_list::create_item,
        shopping_list::patch_item,
        shopping_list::consolidate,
        shopping_list::from_recipe,
        shopping_list::from_plan,
        shopping_list::delete_completed,
        agent::recent_recipes,
        agent::search_recipes,
        agent::weekly_plan,
        agent::add_shopping_items,
        agent::delete_completed,
        agent::create_recipe,
        crate::auth::routes::login,
        crate::auth::routes::callback,
        crate::auth::routes::me,
        crate::auth::routes::logout,
    ),
    components(schemas(
        crate::domain::ingredient::IngredientInput,
        crate::models::recipe::RecipeSummary,
        crate::models::recipe::Recipe,
        crate::models::recipe::RecipeInput,
        crate::routes::recipes::ImportRecipeRequest,
        crate::models::recipe::RecipeQuery,
        crate::models::recipe::RecipeListResponse,
        crate::models::tag::Tag,
        crate::models::tag::TagInput,
        crate::models::tag::RecipeTag,
        crate::models::settings::HouseholdSettings,
        crate::models::settings::SettingsInput,
        crate::models::meal_plan::MealPlanEntryView,
        crate::models::meal_plan::MealPlanInput,
        crate::models::meal_plan::MealPlanPatchInput,
        crate::models::meal_plan::MealPlanQuery,
        crate::models::shopping_list::ShoppingItem,
        crate::models::shopping_list::ShoppingListGroup,
        crate::models::shopping_list::ShoppingItemInput,
        crate::models::shopping_list::ShoppingItemPatch,
        crate::models::shopping_list::ConsolidateInput,
        crate::models::shopping_list::RecipeToShoppingListInput,
        crate::models::shopping_list::PlanToShoppingListInput,
        crate::models::agent::RecentRecipe,
        crate::models::agent::RecentRecipesResponse,
        crate::models::agent::RecentRecipesQuery,
        crate::models::agent::SearchRecipesResponse,
        crate::models::agent::SearchRecipesQuery,
        crate::models::agent::StructuredPlanEntry,
        crate::models::agent::StructuredPlan,
        crate::models::agent::WeeklyPlanResponse,
        crate::models::agent::AgentShoppingItemsInput,
        crate::models::agent::RemovedResponse,
        crate::auth::MeResponse,
        crate::auth::routes::LogoutResponse,
    ))
)]
pub struct ApiDoc;

/// Router que sirve el JSON OpenAPI y la Swagger UI.
///
/// La Swagger UI se sirve con handlers propios (en lugar del router por
/// defecto de `utoipa-swagger-ui`) para que `GET /api/docs` responda 200 sin
/// redirección **y** sus recursos relativos se resuelvan bajo `/api/docs/`.
pub fn router() -> Router<AppState> {
    Router::new()
        .route(OPENAPI_PATH, get(openapi_json))
        .route(DOCS_PATH, get(docs_index))
        .route(&format!("{DOCS_PATH}/{{*rest}}"), get(docs_asset))
}

/// Sirve el documento OpenAPI como JSON.
async fn openapi_json() -> Json<utoipa::openapi::OpenApi> {
    Json(ApiDoc::openapi())
}

/// Sirve el `index.html` de la Swagger UI, inyectando un `<base>` para que los
/// recursos relativos se resuelvan bajo `/api/docs/`.
async fn docs_index() -> Response {
    match serve("", swagger_config()) {
        Ok(Some(file)) => {
            let html = inject_base(&String::from_utf8_lossy(file.bytes.as_ref()));
            let mut headers = HeaderMap::new();
            headers.insert(
                header::CONTENT_TYPE,
                HeaderValue::from_static("text/html; charset=utf-8"),
            );
            (headers, Body::from(html)).into_response()
        }
        Ok(None) => StatusCode::NOT_FOUND.into_response(),
        Err(error) => {
            tracing::error!(error = %error, "no se pudo servir la Swagger UI");
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                "error sirviendo Swagger UI",
            )
                .into_response()
        }
    }
}

/// Sirve un recurso estático de la Swagger UI.
async fn docs_asset(Path(rest): Path<String>) -> Response {
    match serve(&rest, swagger_config()) {
        Ok(Some(file)) => {
            let mut headers = HeaderMap::new();
            let content_type = HeaderValue::from_str(&file.content_type)
                .unwrap_or_else(|_| HeaderValue::from_static("application/octet-stream"));
            headers.insert(header::CONTENT_TYPE, content_type);
            if file.gzpipped {
                headers.insert(header::CONTENT_ENCODING, HeaderValue::from_static("gzip"));
            }
            (headers, Body::from(file.bytes.into_owned())).into_response()
        }
        Ok(None) => StatusCode::NOT_FOUND.into_response(),
        Err(error) => {
            tracing::error!(error = %error, "no se pudo servir un recurso de la Swagger UI");
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                "error sirviendo Swagger UI",
            )
                .into_response()
        }
    }
}

/// Configuración de Swagger UI, construida una sola vez.
fn swagger_config() -> Arc<Config<'static>> {
    static CONFIG: OnceLock<Arc<Config<'static>>> = OnceLock::new();

    Arc::clone(
        CONFIG.get_or_init(|| Arc::new(Config::new([Url::new("OxiMeal API", OPENAPI_PATH)]))),
    )
}

/// Inserta `<base href="/api/docs/">` en el `<head>` del HTML si no existe ya.
fn inject_base(html: &str) -> String {
    const BASE: &str = "<base href=\"/api/docs/\">";

    if html.contains("<base ") {
        return html.to_string();
    }

    match html.find("<head>") {
        Some(position) => {
            let insert_at = position + "<head>".len();
            let mut result = String::with_capacity(html.len() + BASE.len());
            result.push_str(&html[..insert_at]);
            result.push_str(BASE);
            result.push_str(&html[insert_at..]);
            result
        }
        None => html.to_string(),
    }
}
