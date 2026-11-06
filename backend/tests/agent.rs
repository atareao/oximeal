//! Tests de integración de la Agent API (`/api/agent`), el contrato HTTP que
//! consume el asistente Valet.

mod common;

use axum::http::StatusCode;
use chrono::{Duration, Local};
use serde_json::{Value, json};

/// Fecha `YYYY-MM-DD` relativa a hoy (hora local).
fn days_ago(days: i64) -> String {
    (Local::now().date_naive() - Duration::days(days))
        .format("%Y-%m-%d")
        .to_string()
}

/// Crea una receta con un único ingrediente y devuelve su id.
async fn create_recipe(
    state: &oximeal_backend::state::AppState,
    name: &str,
    ingredient_name: &str,
    amount: f64,
    unit: &str,
) -> String {
    let (status, body) = common::request(
        state.clone(),
        "POST",
        "/api/recipes",
        Some(json!({
            "name": name,
            "category": "main",
            "ingredients": [{"name": ingredient_name, "amount": amount, "unit": unit}]
        })),
    )
    .await;
    assert_eq!(status, StatusCode::CREATED);
    body["id"].as_str().expect("id de receta").to_string()
}

/// Crea una entrada del plan directamente vía la API pública.
async fn plan_entry(state: &oximeal_backend::state::AppState, date: &str, recipe_id: &str) {
    let (status, _) = common::request(
        state.clone(),
        "POST",
        "/api/meal-plan",
        Some(json!({
            "date": date,
            "meal_type": "lunch",
            "recipe_id": recipe_id,
            "role": "main"
        })),
    )
    .await;
    assert_eq!(status, StatusCode::CREATED);
}

/// Busca un ítem por nombre en la lista agrupada y devuelve su objeto.
fn find_item<'a>(groups: &'a Value, name: &str) -> Option<&'a Value> {
    groups.as_array()?.iter().find_map(|group| {
        group["items"]
            .as_array()?
            .iter()
            .find(|item| item["name"] == name)
    })
}

// --- GET /api/agent/recent-recipes -----------------------------------------

#[tokio::test]
async fn recent_recipes_returns_planned_ordered_by_last_planned() {
    let state = common::test_state().await;

    let recent = create_recipe(&state, "Crema de calabaza", "Calabaza", 1.0, "ud").await;
    let older = create_recipe(&state, "Lentejas", "Lenteja", 1.0, "ud").await;
    let excluded = create_recipe(&state, "Antigua", "Arroz", 1.0, "ud").await;

    plan_entry(&state, &days_ago(2), &recent).await;
    plan_entry(&state, &days_ago(1), &older).await;
    plan_entry(&state, &days_ago(30), &excluded).await;

    let (status, body) =
        common::request(state, "GET", "/api/agent/recent-recipes?days=7", None).await;
    assert_eq!(status, StatusCode::OK);

    let recipes = body["recipes"].as_array().expect("array recipes");
    assert_eq!(recipes.len(), 2, "se excluye la receta fuera de ventana");
    assert_eq!(recipes[0]["id"], older, "más reciente primero");
    assert_eq!(recipes[0]["last_planned"], days_ago(1));
    assert_eq!(recipes[0]["name"], "Lentejas");
    assert_eq!(recipes[0]["category"], "main");
    assert_eq!(recipes[1]["id"], recent);
    assert_eq!(recipes[1]["last_planned"], days_ago(2));
}

#[tokio::test]
async fn recent_recipes_without_data_is_empty_ok() {
    let state = common::test_state().await;

    let (status, body) = common::request(state, "GET", "/api/agent/recent-recipes", None).await;
    assert_eq!(status, StatusCode::OK);
    assert_eq!(body["recipes"], json!([]));
}

// --- GET /api/agent/search-recipes -----------------------------------------

#[tokio::test]
async fn search_recipes_returns_matches() {
    let state = common::test_state().await;

    create_recipe(&state, "Ensalada de tomate", "Tomate", 2.0, "ud").await;
    create_recipe(&state, "Tarta de queso", "Queso", 200.0, "g").await;

    let (status, body) =
        common::request(state, "GET", "/api/agent/search-recipes?query=tomate", None).await;
    assert_eq!(status, StatusCode::OK);

    let recipes = body["recipes"].as_array().expect("array recipes");
    assert!(
        recipes.iter().any(|r| r["name"] == "Ensalada de tomate"),
        "debe encontrar la receta que contiene el término"
    );
}

#[tokio::test]
async fn search_recipes_short_query_uses_like_fallback() {
    let state = common::test_state().await;

    create_recipe(&state, "Tortilla de patatas", "Patata", 3.0, "ud").await;

    // Query de 2 caracteres: por debajo del mínimo de FTS5 (3) → fallback LIKE.
    let (status, body) =
        common::request(state, "GET", "/api/agent/search-recipes?query=to", None).await;
    assert_eq!(status, StatusCode::OK);
    assert!(
        !body["recipes"].as_array().expect("array").is_empty(),
        "el fallback LIKE debe encontrar por substring"
    );
}

#[tokio::test]
async fn search_recipes_invalid_category_returns_400() {
    let state = common::test_state().await;

    let (status, _) = common::request(
        state,
        "GET",
        "/api/agent/search-recipes?query=tomate&category=inventada",
        None,
    )
    .await;
    assert_eq!(status, StatusCode::BAD_REQUEST);
}

/// Percent-encodes a query value so that `http::Uri` can parse it.
fn encode_query(value: &str) -> String {
    value
        .chars()
        .map(|c| match c {
            'A'..='Z' | 'a'..='z' | '0'..='9' | '-' | '_' | '.' | '~' => c.to_string(),
            other => format!("%{:02X}", other as u32),
        })
        .collect()
}

/// F1 — `search-recipes` no debe devolver 500 ante términos con sintaxis FTS5.
#[tokio::test]
async fn search_recipes_special_characters_never_return_500() {
    let state = common::test_state().await;

    create_recipe(&state, "Ensalada de tomate", "Tomate", 2.0, "ud").await;

    for q in ["tomate AND", "a:b", "(((", "\"\"\"", "foo:bar", "OR"] {
        let uri = format!("/api/agent/search-recipes?query={}", encode_query(q));
        let (status, _) = common::request(state.clone(), "GET", &uri, None).await;
        assert_ne!(
            status,
            StatusCode::INTERNAL_SERVER_ERROR,
            "query={q:?} no debe producir 500"
        );
        assert_eq!(status, StatusCode::OK, "query={q:?} debe responder 200");
    }

    let (status, body) =
        common::request(state, "GET", "/api/agent/search-recipes?query=tomate", None).await;
    assert_eq!(status, StatusCode::OK);
    assert!(
        body["recipes"]
            .as_array()
            .is_some_and(|recipes| !recipes.is_empty()),
        "query=tomate debe seguir encontrando"
    );
}

// --- POST /api/agent/weekly-plan -------------------------------------------

#[tokio::test]
async fn weekly_plan_replaces_day_and_rebuilds_shopping_keeping_manual() {
    let state = common::test_state().await;

    let r1 = create_recipe(&state, "Receta 1", "Tomate", 2.0, "ud").await;
    let r2 = create_recipe(&state, "Receta 2", "Tomate", 3.0, "ud").await;
    let r3 = create_recipe(&state, "Receta 3", "Leche", 1.0, "l").await;
    let day = days_ago(1);

    // Ítem manual que NO debe borrarse al reconstruir la compra.
    let (status, _) = common::request(
        state.clone(),
        "POST",
        "/api/shopping-list",
        Some(json!({"name": "Aceite", "amount": 1.0, "unit": "l"})),
    )
    .await;
    assert_eq!(status, StatusCode::CREATED);

    let (status, body) = common::request(
        state.clone(),
        "POST",
        "/api/agent/weekly-plan",
        Some(json!({"entries": [
            {"date": day, "meal_type": "lunch", "recipe_id": r1, "role": "main", "servings": 2},
            {"date": days_ago(0), "meal_type": "dinner", "recipe_id": r2}
        ]})),
    )
    .await;
    assert_eq!(status, StatusCode::OK);
    assert_eq!(body["created"], 2);

    let tomate = find_item(&body["shopping_list"], "Tomate").expect("tomate consolidado");
    assert_eq!(tomate["amount"], 5.0);
    assert_eq!(tomate["unit"], "ud");
    assert!(
        find_item(&body["shopping_list"], "Aceite").is_some(),
        "el ítem manual se conserva"
    );

    // Reintento: se reemplaza el plan del primer día (r1 → r3) y se refija el
    // segundo, cubriendo ambas fechas en la reconstrucción de la compra.
    let (status, body) = common::request(
        state.clone(),
        "POST",
        "/api/agent/weekly-plan",
        Some(json!({"entries": [
            {"date": day, "meal_type": "lunch", "recipe_id": r3, "role": "main"},
            {"date": days_ago(0), "meal_type": "dinner", "recipe_id": r2, "role": "main"}
        ]})),
    )
    .await;
    assert_eq!(status, StatusCode::OK);
    assert_eq!(body["created"], 2);

    // El día reemplazado ya no tiene Tomate de r1; solo r2 (3 ud) mantiene Tomate.
    let tomate = find_item(&body["shopping_list"], "Tomate").expect("tomate de r2");
    assert_eq!(tomate["amount"], 3.0);
    assert!(
        find_item(&body["shopping_list"], "Leche").is_some(),
        "la leche del nuevo plan entra en la compra"
    );

    // Verificación vía API del plan resultante.
    let (status, plan) = common::request(
        state,
        "GET",
        &format!("/api/meal-plan?start_date={}&end_date={}", day, days_ago(0)),
        None,
    )
    .await;
    assert_eq!(status, StatusCode::OK);
    let entries = plan.as_array().expect("array plan");
    assert_eq!(entries.len(), 2);
    let day_entry = entries
        .iter()
        .find(|e| e["date"] == day)
        .expect("entrada del día reemplazado");
    assert_eq!(day_entry["recipe"]["id"], r3);
}

#[tokio::test]
async fn weekly_plan_unknown_recipe_returns_400_and_persists_nothing() {
    let state = common::test_state().await;

    let valid = create_recipe(&state, "Válida", "Tomate", 1.0, "ud").await;
    let day = days_ago(1);

    let (status, _) = common::request(
        state.clone(),
        "POST",
        "/api/agent/weekly-plan",
        Some(json!({"entries": [
            {"date": day, "meal_type": "lunch", "recipe_id": valid},
            {"date": days_ago(0), "meal_type": "dinner", "recipe_id": "no-existe"}
        ]})),
    )
    .await;
    assert_eq!(status, StatusCode::BAD_REQUEST);

    let (_, plan) = common::request(
        state.clone(),
        "GET",
        &format!("/api/meal-plan?start_date={}&end_date={}", day, days_ago(0)),
        None,
    )
    .await;
    assert!(
        plan.as_array().expect("array plan").is_empty(),
        "no debe persistirse ninguna entrada"
    );

    let (_, groups) = common::request(state, "GET", "/api/shopping-list", None).await;
    assert!(
        groups.as_array().expect("array lista").is_empty(),
        "la compra no debe reconstruirse si falla la validación"
    );
}

#[tokio::test]
async fn weekly_plan_invalid_enum_returns_400() {
    let state = common::test_state().await;

    let recipe = create_recipe(&state, "Receta", "Tomate", 1.0, "ud").await;

    let (status, _) = common::request(
        state,
        "POST",
        "/api/agent/weekly-plan",
        Some(json!({"entries": [
            {"date": days_ago(1), "meal_type": "brunch", "recipe_id": recipe}
        ]})),
    )
    .await;
    assert_eq!(status, StatusCode::BAD_REQUEST);
}

/// F2 — `weekly-plan` debe normalizar la fecha al persistir para que los rangos
/// `WHERE date >= ... <= ...` la encuentren.
#[tokio::test]
async fn weekly_plan_persists_canonical_dates() {
    let state = common::test_state().await;
    let recipe = create_recipe(&state, "Receta", "Tomate", 1.0, "ud").await;

    let (status, _) = common::request(
        state.clone(),
        "POST",
        "/api/agent/weekly-plan",
        Some(json!({"entries": [
            {"date": "2026-1-2", "meal_type": "lunch", "recipe_id": recipe, "role": "main"}
        ]})),
    )
    .await;
    assert_eq!(status, StatusCode::OK);

    let (status, plan) = common::request(
        state,
        "GET",
        "/api/meal-plan?start_date=2026-01-01&end_date=2026-01-31",
        None,
    )
    .await;
    assert_eq!(status, StatusCode::OK);
    let entries = plan.as_array().expect("array plan");
    assert_eq!(entries.len(), 1, "la entrada debe aparecer en el rango");
    assert_eq!(entries[0]["date"], "2026-01-02");
}

// --- POST /api/agent/shopping-items ----------------------------------------

#[tokio::test]
async fn shopping_items_sums_existing_manual_item() {
    let state = common::test_state().await;

    let (status, _) = common::request(
        state.clone(),
        "POST",
        "/api/shopping-list",
        Some(json!({"name": "Tomate", "amount": 2.0, "unit": "ud"})),
    )
    .await;
    assert_eq!(status, StatusCode::CREATED);

    // "tomates" normaliza a la misma clave "tomate"; misma unidad canónica "ud".
    let (status, groups) = common::request(
        state.clone(),
        "POST",
        "/api/agent/shopping-items",
        Some(json!({"items": [{"name": "tomates", "amount": 3.0, "unit": "ud"}]})),
    )
    .await;
    assert_eq!(status, StatusCode::OK);

    let item = find_item(&groups, "Tomate").expect("ítem existente");
    assert_eq!(item["amount"], 5.0, "las cantidades se suman");

    let (_, groups) = common::request(state, "GET", "/api/shopping-list", None).await;
    let item = find_item(&groups, "Tomate").expect("ítem persistido");
    assert_eq!(item["amount"], 5.0);
}

#[tokio::test]
async fn shopping_items_inserts_new_with_heuristic_category() {
    let state = common::test_state().await;

    let (status, groups) = common::request(
        state,
        "POST",
        "/api/agent/shopping-items",
        Some(json!({"items": [{"name": "Merluza", "amount": 1.0, "unit": "kg"}]})),
    )
    .await;
    assert_eq!(status, StatusCode::OK);

    let item = find_item(&groups, "Merluza").expect("ítem nuevo");
    assert_eq!(item["category"], "Pescadería");
    assert_eq!(item["unit"], "g", "la unidad se canonicaliza");
    assert_eq!(item["amount"], 1000.0);
}

#[tokio::test]
async fn shopping_items_empty_name_returns_400() {
    let state = common::test_state().await;

    let (status, _) = common::request(
        state,
        "POST",
        "/api/agent/shopping-items",
        Some(json!({"items": [{"name": "  ", "amount": 1.0, "unit": "ud"}]})),
    )
    .await;
    assert_eq!(status, StatusCode::BAD_REQUEST);
}

/// F10 — Para familia `Unknown` la clave de conciliación debe normalizar la
/// unidad: `"Cajas"` y `"cajas"` son la misma unidad y se suman.
#[tokio::test]
async fn shopping_items_reconciles_unknown_unit_case_insensitively() {
    let state = common::test_state().await;

    let (status, groups) = common::request(
        state.clone(),
        "POST",
        "/api/agent/shopping-items",
        Some(json!({"items": [
            {"name": "Cajas", "amount": 1.0, "unit": "Cajas"},
            {"name": "Cajas", "amount": 1.0, "unit": "cajas"}
        ]})),
    )
    .await;
    assert_eq!(status, StatusCode::OK);

    let item = find_item(&groups, "Cajas").expect("ítem consolidado");
    assert_eq!(
        item["amount"], 2.0,
        "las unidades con distinta caja se reconcilian y suman"
    );

    let (_, groups) = common::request(state, "GET", "/api/shopping-list", None).await;
    let all: Vec<&Value> = groups
        .as_array()
        .expect("array de grupos")
        .iter()
        .flat_map(|group| group["items"].as_array().into_iter().flatten())
        .collect();
    assert_eq!(all.len(), 1, "no debe haber dos ítems separados");
    assert_eq!(all[0]["amount"], 2.0);
}

// --- DELETE /api/agent/shopping/completed ----------------------------------

#[tokio::test]
async fn delete_completed_removes_checked_and_reports_count() {
    let state = common::test_state().await;

    let (_, a) = common::request(
        state.clone(),
        "POST",
        "/api/shopping-list",
        Some(json!({"name": "Tomate", "amount": 1.0, "unit": "ud"})),
    )
    .await;
    let (_, b) = common::request(
        state.clone(),
        "POST",
        "/api/shopping-list",
        Some(json!({"name": "Leche", "amount": 1.0, "unit": "l"})),
    )
    .await;
    let id_a = a["id"].as_str().expect("id a").to_string();
    let id_b = b["id"].as_str().expect("id b").to_string();

    let (status, _) = common::request(
        state.clone(),
        "PATCH",
        &format!("/api/shopping-list/{id_a}"),
        Some(json!({"checked": true})),
    )
    .await;
    assert_eq!(status, StatusCode::OK);

    let (status, body) = common::request(
        state.clone(),
        "DELETE",
        "/api/agent/shopping/completed",
        None,
    )
    .await;
    assert_eq!(status, StatusCode::OK);
    assert_eq!(body["removed"], 1);

    let (_, groups) = common::request(state, "GET", "/api/shopping-list", None).await;
    assert!(
        find_item(&groups, "Tomate").is_none(),
        "el marcado se elimina"
    );
    assert!(
        find_item(&groups, "Leche").is_some(),
        "el no marcado permanece"
    );
    assert!(!id_b.is_empty());
}

// --- POST /api/agent/recipes -----------------------------------------------

#[tokio::test]
async fn create_recipe_returns_201_and_persists() {
    let state = common::test_state().await;

    let (status, body) = common::request(
        state.clone(),
        "POST",
        "/api/agent/recipes",
        Some(json!({
            "name": "Tortilla",
            "category": "main",
            "ingredients": [{"name": "Huevo", "amount": 3.0, "unit": "ud"}]
        })),
    )
    .await;
    assert_eq!(status, StatusCode::CREATED, "body: {body}");

    let id = body["id"].as_str().expect("id de receta").to_string();
    assert_eq!(body["name"], "Tortilla");

    let (status, detail) = common::request(state, "GET", &format!("/api/recipes/{id}"), None).await;
    assert_eq!(status, StatusCode::OK, "la receta debe persistirse");
    assert_eq!(detail["id"], id);
    assert_eq!(detail["name"], "Tortilla");
}

#[tokio::test]
async fn create_recipe_rejects_invalid_category() {
    let state = common::test_state().await;

    let (status, _) = common::request(
        state,
        "POST",
        "/api/agent/recipes",
        Some(json!({
            "name": "Tarta",
            "category": "postre",
            "ingredients": [{"name": "Huevo", "amount": 3.0, "unit": "ud"}]
        })),
    )
    .await;
    assert_eq!(status, StatusCode::BAD_REQUEST);
}

#[tokio::test]
async fn create_recipe_rejects_empty_ingredients() {
    let state = common::test_state().await;

    let (status, _) = common::request(
        state,
        "POST",
        "/api/agent/recipes",
        Some(json!({
            "name": "Tortilla",
            "category": "main",
            "ingredients": []
        })),
    )
    .await;
    assert_eq!(status, StatusCode::BAD_REQUEST);
}

// --- Documentación OpenAPI -------------------------------------------------

#[tokio::test]
async fn agent_paths_are_documented_in_openapi() {
    let state = common::test_state().await;

    let (status, body) = common::request(state, "GET", "/api/openapi.json", None).await;
    assert_eq!(status, StatusCode::OK);

    let paths = body["paths"].as_object().expect("objeto paths");
    for path in [
        "/api/agent/recent-recipes",
        "/api/agent/search-recipes",
        "/api/agent/weekly-plan",
        "/api/agent/shopping-items",
        "/api/agent/shopping/completed",
    ] {
        assert!(paths.contains_key(path), "falta {path} en paths");
    }
}
