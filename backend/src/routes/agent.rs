//! Endpoints de la Agent API: contrato HTTP estable que consume el asistente
//! Valet. OxiMeal no orquesta LLM ni expone chat: solo estas operaciones
//! tipadas sobre recetas, plan y lista de la compra.

use axum::Json;
use axum::Router;
use axum::extract::{Query, State};
use axum::routing::{delete, get, post};

use crate::error::{AppError, AppResult};
use crate::models::agent::{
    AgentShoppingItemsInput, RecentRecipesQuery, RecentRecipesResponse, RemovedResponse,
    SearchRecipesQuery, SearchRecipesResponse, StructuredPlan, WeeklyPlanResponse,
};
use crate::models::shopping_list::ShoppingListGroup;
use crate::repo;
use crate::routes::validation;
use crate::state::AppState;

/// Máximo de resultados que la búsqueda del agente devuelve.
const SEARCH_LIMIT: u32 = 10;
/// Ventana por defecto de `recent-recipes`, en días.
const DEFAULT_RECENT_DAYS: u32 = 7;
/// Ventana máxima admitida de `recent-recipes`, en días.
const MAX_RECENT_DAYS: u32 = 365;

/// Router de la Agent API de lectura (permiso `agent:read`).
pub fn read_router() -> Router<AppState> {
    Router::new()
        .route("/api/agent/recent-recipes", get(recent_recipes))
        .route("/api/agent/search-recipes", get(search_recipes))
}

/// Router de la Agent API de planificación (permiso `agent:plan`).
pub fn plan_router() -> Router<AppState> {
    Router::new().route("/api/agent/weekly-plan", post(weekly_plan))
}

/// Router de la Agent API de lista de la compra (permiso `agent:shopping`).
pub fn shopping_router() -> Router<AppState> {
    Router::new()
        .route("/api/agent/shopping-items", post(add_shopping_items))
        .route("/api/agent/shopping/completed", delete(delete_completed))
}

/// `GET /api/agent/recent-recipes` — recetas planificadas recientemente.
#[utoipa::path(
    get,
    path = "/api/agent/recent-recipes",
    tag = "agent",
    params(
        ("days" = Option<u32>, Query, description = "Ventana en días hacia atrás (default 7, máx 365)"),
    ),
    responses(
        (status = 200, description = "Recetas planificadas, más recientes primero", body = RecentRecipesResponse),
    )
)]
pub async fn recent_recipes(
    State(state): State<AppState>,
    Query(query): Query<RecentRecipesQuery>,
) -> AppResult<Json<RecentRecipesResponse>> {
    let days = query
        .days
        .unwrap_or(DEFAULT_RECENT_DAYS)
        .clamp(1, MAX_RECENT_DAYS);

    let recipes = repo::agent::recent_recipes(&state.pool, days).await?;
    Ok(Json(RecentRecipesResponse { recipes }))
}

/// `GET /api/agent/search-recipes` — búsqueda de recetas para el agente.
#[utoipa::path(
    get,
    path = "/api/agent/search-recipes",
    tag = "agent",
    params(
        ("query" = Option<String>, Query, description = "Texto de búsqueda (FTS5 si >= 3 caracteres)"),
        ("category" = Option<String>, Query, description = "Filtro opcional por categoría"),
    ),
    responses(
        (status = 200, description = "Coincidencias (máx 10)", body = SearchRecipesResponse),
        (status = 400, description = "Categoría inválida"),
    )
)]
pub async fn search_recipes(
    State(state): State<AppState>,
    Query(query): Query<SearchRecipesQuery>,
) -> AppResult<Json<SearchRecipesResponse>> {
    let search = query
        .query
        .as_deref()
        .map(str::trim)
        .filter(|value| !value.is_empty());
    let category = query
        .category
        .as_deref()
        .map(str::trim)
        .filter(|value| !value.is_empty());

    if let Some(category) = category
        && !validation::is_valid_category(category)
    {
        return Err(AppError::Validation(format!(
            "categoría inválida: {category}"
        )));
    }

    let response = repo::recipes::list(&state.pool, search, category, 1, SEARCH_LIMIT).await?;
    Ok(Json(SearchRecipesResponse {
        recipes: response.items,
    }))
}

/// `POST /api/agent/weekly-plan` — fija el plan semanal y reconstruye la compra.
#[utoipa::path(
    post,
    path = "/api/agent/weekly-plan",
    tag = "agent",
    request_body = StructuredPlan,
    responses(
        (status = 200, description = "Plan fijado y compra reconstruida", body = WeeklyPlanResponse),
        (status = 400, description = "Entrada inválida o receta inexistente"),
    )
)]
pub async fn weekly_plan(
    State(state): State<AppState>,
    Json(plan): Json<StructuredPlan>,
) -> AppResult<Json<WeeklyPlanResponse>> {
    for entry in &plan.entries {
        validation::parse_date(&entry.date)?;
        if !validation::is_valid_meal_type(&entry.meal_type) {
            return Err(AppError::Validation(format!(
                "meal_type inválido: {}",
                entry.meal_type
            )));
        }
        if !validation::is_valid_role(&entry.role) {
            return Err(AppError::Validation(format!(
                "role inválido: {}",
                entry.role
            )));
        }
        if entry.recipe_id.trim().is_empty() {
            return Err(AppError::Validation("recipe_id es obligatorio".into()));
        }
        if entry.servings < 1 {
            return Err(AppError::Validation("servings debe ser >= 1".into()));
        }
    }

    let (created, shopping_list) = repo::agent::set_weekly_plan(&state.pool, &plan.entries).await?;
    Ok(Json(WeeklyPlanResponse {
        created,
        shopping_list,
    }))
}

/// `POST /api/agent/shopping-items` — añade o suma ingredientes a la compra.
#[utoipa::path(
    post,
    path = "/api/agent/shopping-items",
    tag = "agent",
    request_body = AgentShoppingItemsInput,
    responses(
        (status = 200, description = "Lista de la compra resultante", body = [ShoppingListGroup]),
        (status = 400, description = "Ítem inválido"),
    )
)]
pub async fn add_shopping_items(
    State(state): State<AppState>,
    Json(input): Json<AgentShoppingItemsInput>,
) -> AppResult<Json<Vec<ShoppingListGroup>>> {
    for item in &input.items {
        if item.name.trim().is_empty() {
            return Err(AppError::Validation(
                "el nombre no puede estar vacío".into(),
            ));
        }
        if item.amount < 0.0 {
            return Err(AppError::Validation("amount debe ser >= 0".into()));
        }
    }

    let groups = repo::agent::add_shopping_items(&state.pool, &input.items).await?;
    Ok(Json(groups))
}

/// `DELETE /api/agent/shopping/completed` — purga los ítems ya comprados.
#[utoipa::path(
    delete,
    path = "/api/agent/shopping/completed",
    tag = "agent",
    responses(
        (status = 200, description = "Número de ítems eliminados", body = RemovedResponse),
    )
)]
pub async fn delete_completed(State(state): State<AppState>) -> AppResult<Json<RemovedResponse>> {
    let removed = repo::agent::delete_completed(&state.pool).await?;
    Ok(Json(RemovedResponse { removed }))
}
