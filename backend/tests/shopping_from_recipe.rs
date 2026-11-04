//! Tests de integración del volcado de una receta a la lista de la compra
//! (`POST /api/shopping-list/from-recipe`).
//!
//! Source of truth: `openspec/changes/shopping-from-recipe/specs/`.

mod common;

use axum::body::Body;
use axum::http::{Request, StatusCode, header};
use common::idp::{API_AUDIENCE, MockIdp, USER_GROUPS, access_token};
use common::{send, test_state, test_state_oidc};
use oximeal_backend::state::AppState;
use serde_json::{Value, json};

/// Crea una receta con raciones e ingredientes y devuelve su id.
async fn create_recipe(state: &AppState, name: &str, servings: i64, ingredients: Value) -> String {
    let (status, body) = common::request(
        state.clone(),
        "POST",
        "/api/recipes",
        Some(json!({
            "name": name,
            "category": "main",
            "servings": servings,
            "ingredients": ingredients,
        })),
    )
    .await;
    assert_eq!(status, StatusCode::CREATED, "crear receta: {body}");
    body["id"].as_str().expect("id de receta").to_string()
}

/// Aplana todos los ítems de la lista agrupada.
fn all_items(groups: &Value) -> Vec<&Value> {
    groups
        .as_array()
        .into_iter()
        .flatten()
        .flat_map(|group| group["items"].as_array().into_iter().flatten())
        .collect()
}

/// Busca un ítem por nombre de presentación exacto.
fn find_item<'a>(groups: &'a Value, name: &str) -> Option<&'a Value> {
    all_items(groups)
        .into_iter()
        .find(|item| item["name"] == name)
}

/// Devuelve `(source_plan_id, source_recipe_id)` de la línea con ese nombre.
async fn item_sources(state: &AppState, name: &str) -> (Option<String>, Option<String>) {
    sqlx::query_as::<_, (Option<String>, Option<String>)>(
        "SELECT source_plan_id, source_recipe_id FROM shopping_list_items WHERE name = ?",
    )
    .bind(name)
    .fetch_one(&state.pool)
    .await
    .expect("leer fuentes de la línea")
}

/// Cuenta las líneas derivadas de recetas (`source_recipe_id` no nulo).
async fn recipe_derived_count(state: &AppState) -> i64 {
    sqlx::query_scalar(
        "SELECT count(*) FROM shopping_list_items WHERE source_recipe_id IS NOT NULL",
    )
    .fetch_one(&state.pool)
    .await
    .expect("contar líneas derivadas de recetas")
}

/// Da de alta un ítem manual y comprueba que se creó.
async fn add_manual_item(state: &AppState, name: &str, amount: f64, unit: &str) {
    let (status, _) = common::request(
        state.clone(),
        "POST",
        "/api/shopping-list",
        Some(json!({"name": name, "amount": amount, "unit": unit})),
    )
    .await;
    assert_eq!(status, StatusCode::CREATED, "alta manual de {name}");
}

/// Petición con Bearer opcional para los escenarios de permisos.
async fn call(
    state: &AppState,
    uri: &str,
    bearer: Option<&str>,
    body: Option<Value>,
) -> StatusCode {
    let mut builder = Request::builder().method("POST").uri(uri);
    if let Some(token) = bearer {
        builder = builder.header(header::AUTHORIZATION, format!("Bearer {token}"));
    }
    let body = match body {
        Some(value) => {
            builder = builder.header(header::CONTENT_TYPE, "application/json");
            Body::from(serde_json::to_vec(&value).expect("serializar cuerpo"))
        }
        None => Body::empty(),
    };
    let (status, _headers, _value) = send(
        state.clone(),
        builder.body(body).expect("construir request"),
    )
    .await;
    status
}

/// Escenario: Volcado feliz — líneas escaladas y marcadas con la receta fuente.
#[tokio::test]
async fn from_recipe_dumps_scaled_lines_marked_with_source_recipe() {
    let state = test_state().await;

    let recipe = create_recipe(
        &state,
        "Arroz con tomate",
        2,
        json!([{"name": "Arroz", "amount": 100.0, "unit": "g"}]),
    )
    .await;

    let (status, groups) = common::request(
        state.clone(),
        "POST",
        "/api/shopping-list/from-recipe",
        Some(json!({"recipe_id": recipe, "servings": 4})),
    )
    .await;
    assert_eq!(status, StatusCode::OK, "respuesta: {groups}");

    let arroz = find_item(&groups, "Arroz").expect("arroz volcado");
    assert_eq!(arroz["amount"], 200.0, "2 -> 4 raciones duplica");
    assert_eq!(arroz["unit"], "g");
    assert_eq!(arroz["checked"], false);

    let (_, source_recipe_id) = item_sources(&state, "Arroz").await;
    assert_eq!(
        source_recipe_id.as_deref(),
        Some(recipe.as_str()),
        "la línea debe quedar marcada con la receta fuente"
    );
}

/// Escenario: Contrato de la petición — sin `servings` es válido (raciones por defecto).
#[tokio::test]
async fn from_recipe_without_servings_uses_recipe_servings() {
    let state = test_state().await;

    let recipe = create_recipe(
        &state,
        "Receta base",
        2,
        json!([{"name": "Arroz", "amount": 100.0, "unit": "g"}]),
    )
    .await;

    let (status, groups) = common::request(
        state.clone(),
        "POST",
        "/api/shopping-list/from-recipe",
        Some(json!({"recipe_id": recipe})),
    )
    .await;
    assert_eq!(status, StatusCode::OK, "respuesta: {groups}");

    let arroz = find_item(&groups, "Arroz").expect("arroz volcado");
    assert_eq!(arroz["amount"], 100.0, "sin servings el factor es 1");
}

/// Escenario: Raciones inválidas — 0, negativo o no entero devuelven 400 sin tocar la lista.
#[tokio::test]
async fn from_recipe_invalid_servings_returns_400_without_modifying_list() {
    let state = test_state().await;
    let recipe = create_recipe(
        &state,
        "Receta base",
        2,
        json!([{"name": "Arroz", "amount": 100.0, "unit": "g"}]),
    )
    .await;
    add_manual_item(&state, "Leche", 1.0, "l").await;

    for servings in [json!(0), json!(-1), json!(2.5), json!("dos")] {
        let (status, _) = common::request(
            state.clone(),
            "POST",
            "/api/shopping-list/from-recipe",
            Some(json!({"recipe_id": recipe, "servings": servings})),
        )
        .await;
        assert_eq!(
            status,
            StatusCode::BAD_REQUEST,
            "servings inválido {servings} debe dar 400"
        );
    }

    // La lista no se ha modificado: ni líneas de receta ni pérdida del manual.
    assert_eq!(
        recipe_derived_count(&state).await,
        0,
        "sin líneas de receta"
    );
    let (status, groups) = common::request(state, "GET", "/api/shopping-list", None).await;
    assert_eq!(status, StatusCode::OK);
    assert!(find_item(&groups, "Leche").is_some(), "el manual permanece");
    assert!(
        find_item(&groups, "Arroz").is_none(),
        "no debe crearse la línea de receta"
    );
}

/// Escenario: Receta inexistente — 404 sin tocar la lista.
#[tokio::test]
async fn from_recipe_unknown_recipe_returns_404_without_modifying_list() {
    let state = test_state().await;
    add_manual_item(&state, "Leche", 1.0, "l").await;

    let (status, _) = common::request(
        state.clone(),
        "POST",
        "/api/shopping-list/from-recipe",
        Some(json!({"recipe_id": "no-existe", "servings": 2})),
    )
    .await;
    assert_eq!(status, StatusCode::NOT_FOUND);

    assert_eq!(
        recipe_derived_count(&state).await,
        0,
        "sin líneas de receta"
    );
    let (_, groups) = common::request(state, "GET", "/api/shopping-list", None).await;
    assert!(find_item(&groups, "Leche").is_some(), "el manual permanece");
}

/// Escenario: Lista global única — manuales y líneas del plan permanecen.
#[tokio::test]
async fn from_recipe_keeps_manual_and_plan_lines_in_single_global_list() {
    let state = test_state().await;

    add_manual_item(&state, "Leche", 1.0, "l").await;

    // Línea derivada del plan.
    let plan_recipe = create_recipe(
        &state,
        "Receta del plan",
        2,
        json!([{"name": "Tomate", "amount": 2.0, "unit": "ud"}]),
    )
    .await;
    let (status, _) = common::request(
        state.clone(),
        "POST",
        "/api/meal-plan",
        Some(json!({
            "date": "2026-03-02", "meal_type": "lunch",
            "recipe_id": plan_recipe, "role": "main"
        })),
    )
    .await;
    assert_eq!(status, StatusCode::CREATED);
    let (status, _) = common::request(
        state.clone(),
        "POST",
        "/api/shopping-list/consolidate",
        Some(json!({"start_date": "2026-03-01", "end_date": "2026-03-07"})),
    )
    .await;
    assert_eq!(status, StatusCode::OK);

    // Volcado de otra receta.
    let recipe = create_recipe(
        &state,
        "Receta volcada",
        2,
        json!([{"name": "Arroz", "amount": 100.0, "unit": "g"}]),
    )
    .await;
    let (status, groups) = common::request(
        state.clone(),
        "POST",
        "/api/shopping-list/from-recipe",
        Some(json!({"recipe_id": recipe, "servings": 2})),
    )
    .await;
    assert_eq!(status, StatusCode::OK, "respuesta: {groups}");

    assert!(find_item(&groups, "Leche").is_some(), "el manual permanece");
    assert!(
        find_item(&groups, "Tomate").is_some(),
        "la línea del plan permanece"
    );
    assert!(
        find_item(&groups, "Arroz").is_some(),
        "la línea de receta se añade"
    );

    let (plan_source, _) = item_sources(&state, "Tomate").await;
    assert_eq!(
        plan_source.as_deref(),
        Some("consolidated"),
        "el tomate viene del plan"
    );
    let (_, recipe_source) = item_sources(&state, "Arroz").await;
    assert_eq!(recipe_source.as_deref(), Some(recipe.as_str()));
}

/// Vista semántica de la lista: ignora los identificadores regenerados en cada
/// reconstrucción declarativa (el contrato es el contenido, no las filas físicas).
fn semantic_view(groups: &Value) -> Vec<(String, String, String, f64)> {
    let mut view: Vec<(String, String, String, f64)> = all_items(groups)
        .into_iter()
        .map(|item| {
            (
                item["name"].as_str().unwrap_or_default().to_string(),
                item["unit"].as_str().unwrap_or_default().to_string(),
                item["category"].as_str().unwrap_or_default().to_string(),
                item["amount"].as_f64().unwrap_or_default(),
            )
        })
        .collect();
    view.sort_by(|a, b| a.0.cmp(&b.0).then_with(|| a.1.cmp(&b.1)));
    view
}

/// Escenario: Repetición idempotente — no duplica líneas.
#[tokio::test]
async fn from_recipe_is_idempotent_on_repeat() {
    let state = test_state().await;
    let recipe = create_recipe(
        &state,
        "Receta base",
        2,
        json!([{"name": "Arroz", "amount": 100.0, "unit": "g"}]),
    )
    .await;

    let payload = json!({"recipe_id": recipe, "servings": 4});
    let (first_status, first) = common::request(
        state.clone(),
        "POST",
        "/api/shopping-list/from-recipe",
        Some(payload.clone()),
    )
    .await;
    assert_eq!(first_status, StatusCode::OK);

    let (status, second) = common::request(
        state.clone(),
        "POST",
        "/api/shopping-list/from-recipe",
        Some(payload),
    )
    .await;
    assert_eq!(status, StatusCode::OK);
    assert_eq!(
        semantic_view(&first),
        semantic_view(&second),
        "repetir la petición produce la misma lista"
    );

    let count: i64 =
        sqlx::query_scalar("SELECT count(*) FROM shopping_list_items WHERE source_recipe_id = ?")
            .bind(&recipe)
            .fetch_one(&state.pool)
            .await
            .expect("contar líneas de la receta");
    assert_eq!(count, 1, "no debe duplicar la línea");

    assert_eq!(find_item(&second, "Arroz").expect("arroz")["amount"], 200.0);
}

/// Escenario: Cambio de raciones reescala sin sumar la cantidad anterior.
#[tokio::test]
async fn from_recipe_rescales_on_servings_change() {
    let state = test_state().await;
    let recipe = create_recipe(
        &state,
        "Receta base",
        2,
        json!([{"name": "Arroz", "amount": 100.0, "unit": "g"}]),
    )
    .await;

    let (status, groups) = common::request(
        state.clone(),
        "POST",
        "/api/shopping-list/from-recipe",
        Some(json!({"recipe_id": recipe, "servings": 4})),
    )
    .await;
    assert_eq!(status, StatusCode::OK);
    assert_eq!(find_item(&groups, "Arroz").expect("arroz")["amount"], 200.0);

    let (status, groups) = common::request(
        state.clone(),
        "POST",
        "/api/shopping-list/from-recipe",
        Some(json!({"recipe_id": recipe, "servings": 2})),
    )
    .await;
    assert_eq!(status, StatusCode::OK);
    assert_eq!(
        find_item(&groups, "Arroz").expect("arroz")["amount"],
        100.0,
        "no permanece sumada la cantidad anterior"
    );
}

/// Escenario: Fusión entre recetas — mismo nombre canónico y unidad se suman.
#[tokio::test]
async fn from_recipe_merges_two_recipes_on_same_canonical_name() {
    let state = test_state().await;

    let r1 = create_recipe(
        &state,
        "Receta 1",
        2,
        json!([{"name": "Tomate", "amount": 2.0, "unit": "ud"}]),
    )
    .await;
    let r2 = create_recipe(
        &state,
        "Receta 2",
        2,
        json!([{"name": "Tomates", "amount": 3.0, "unit": "ud"}]),
    )
    .await;

    for recipe in [&r1, &r2] {
        let (status, _) = common::request(
            state.clone(),
            "POST",
            "/api/shopping-list/from-recipe",
            Some(json!({"recipe_id": recipe, "servings": 2})),
        )
        .await;
        assert_eq!(status, StatusCode::OK);
    }

    let (_, groups) = common::request(state.clone(), "GET", "/api/shopping-list", None).await;
    let tomate_lines: Vec<&Value> = all_items(&groups)
        .into_iter()
        .filter(|item| item["unit"] == "ud")
        .collect();
    assert_eq!(tomate_lines.len(), 1, "una sola línea de tomate fusionada");
    assert_eq!(tomate_lines[0]["amount"], 5.0, "2 + 3 ud sumadas");

    let name = tomate_lines[0]["name"].as_str().expect("nombre");
    let (_, source_recipe_id) = item_sources(&state, name).await;
    assert_eq!(
        source_recipe_id.as_deref(),
        Some("consolidated"),
        "una línea que fusiona varias recetas se marca como consolidated"
    );
}

/// Escenario: No aditivos separados — "al gusto" y "1 pizca" no se suman.
#[tokio::test]
async fn from_recipe_keeps_non_additive_lines_separate() {
    let state = test_state().await;

    let r1 = create_recipe(
        &state,
        "Receta salada",
        2,
        json!([{"name": "Sal", "amount": 0.0, "unit": "al gusto"}]),
    )
    .await;
    let r2 = create_recipe(
        &state,
        "Receta con pizca",
        2,
        json!([{"name": "Sal", "amount": 1.0, "unit": "pizca"}]),
    )
    .await;

    for recipe in [&r1, &r2] {
        let (status, _) = common::request(
            state.clone(),
            "POST",
            "/api/shopping-list/from-recipe",
            Some(json!({"recipe_id": recipe, "servings": 2})),
        )
        .await;
        assert_eq!(status, StatusCode::OK);
    }

    let (status, groups) = common::request(state, "GET", "/api/shopping-list", None).await;
    assert_eq!(status, StatusCode::OK);

    let sal_lines: Vec<&Value> = all_items(&groups)
        .into_iter()
        .filter(|item| item["name"] == "Sal")
        .collect();
    assert_eq!(sal_lines.len(), 2, "la sal queda en dos líneas separadas");

    let to_taste = sal_lines
        .iter()
        .find(|item| item["unit"] == "al gusto")
        .expect("sal al gusto");
    assert_eq!(to_taste["amount"], 0.0);
    let pinch = sal_lines
        .iter()
        .find(|item| item["unit"] == "pizca")
        .expect("sal 1 pizca");
    assert_eq!(pinch["amount"], 1.0, "no se suma con el 'al gusto'");
}

/// Escenario: Familias incompatibles separadas — 500 ml vs 1 ud de leche.
#[tokio::test]
async fn from_recipe_keeps_incompatible_families_separate() {
    let state = test_state().await;
    let recipe = create_recipe(
        &state,
        "Receta láctea",
        2,
        json!([
            {"name": "Leche", "amount": 500.0, "unit": "ml"},
            {"name": "Leche", "amount": 1.0, "unit": "ud"}
        ]),
    )
    .await;

    let (status, groups) = common::request(
        state,
        "POST",
        "/api/shopping-list/from-recipe",
        Some(json!({"recipe_id": recipe, "servings": 2})),
    )
    .await;
    assert_eq!(status, StatusCode::OK);

    let leche: Vec<&Value> = all_items(&groups)
        .into_iter()
        .filter(|item| item["name"] == "Leche")
        .collect();
    assert_eq!(leche.len(), 2, "familias incompatibles → dos líneas");
    assert!(leche.iter().any(|item| item["unit"] == "ml"));
    assert!(leche.iter().any(|item| item["unit"] == "ud"));
}

/// Escenario: Raciones de la receta NULL — factor 1 y se asumen 2.
#[tokio::test]
async fn from_recipe_treats_null_recipe_servings_as_two() {
    let state = test_state().await;

    // La API nunca persiste NULL: se inserta con SQL directo para ejercitar el
    // COALESCE de `recipes.servings`.
    sqlx::query(
        "INSERT INTO recipes (id, name, category, ingredients, servings) \
         VALUES ('rn', 'Receta NULL', 'main', ?, NULL)",
    )
    .bind(r#"[{"name":"Arroz","amount":100.0,"unit":"g"}]"#)
    .execute(&state.pool)
    .await
    .expect("insertar receta NULL");

    let (status, groups) = common::request(
        state.clone(),
        "POST",
        "/api/shopping-list/from-recipe",
        Some(json!({"recipe_id": "rn"})),
    )
    .await;
    assert_eq!(status, StatusCode::OK, "respuesta: {groups}");

    let arroz = find_item(&groups, "Arroz").expect("arroz volcado");
    assert_eq!(arroz["amount"], 100.0, "NULL -> 2, factor 1");

    let stored: i64 =
        sqlx::query_scalar("SELECT servings FROM shopping_recipe_sources WHERE recipe_id = 'rn'")
            .fetch_one(&state.pool)
            .await
            .expect("leer raciones registradas");
    assert_eq!(stored, 2, "las raciones de la receta NULL se asumen 2");
}

/// Escenario: El volcado de receta sobrevive a la consolidación desde el plan.
#[tokio::test]
async fn consolidate_keeps_recipe_derived_lines_and_manual_items() {
    let state = test_state().await;

    // Línea derivada de receta.
    let recipe = create_recipe(
        &state,
        "Receta volcada",
        2,
        json!([{"name": "Arroz", "amount": 100.0, "unit": "g"}]),
    )
    .await;
    let (status, _) = common::request(
        state.clone(),
        "POST",
        "/api/shopping-list/from-recipe",
        Some(json!({"recipe_id": recipe, "servings": 4})),
    )
    .await;
    assert_eq!(status, StatusCode::OK);

    // Ítem manual.
    add_manual_item(&state, "Leche", 1.0, "l").await;

    // Plan semanal.
    let plan_recipe = create_recipe(
        &state,
        "Receta del plan",
        2,
        json!([{"name": "Tomate", "amount": 2.0, "unit": "ud"}]),
    )
    .await;
    let (status, _) = common::request(
        state.clone(),
        "POST",
        "/api/meal-plan",
        Some(json!({
            "date": "2026-03-02", "meal_type": "lunch",
            "recipe_id": plan_recipe, "role": "main"
        })),
    )
    .await;
    assert_eq!(status, StatusCode::CREATED);

    let (status, groups) = common::request(
        state.clone(),
        "POST",
        "/api/shopping-list/consolidate",
        Some(json!({"start_date": "2026-03-01", "end_date": "2026-03-07"})),
    )
    .await;
    assert_eq!(status, StatusCode::OK, "respuesta: {groups}");

    // La línea de receta permanece intacta (200 g, source_recipe_id = receta).
    let arroz = find_item(&groups, "Arroz").expect("arroz de receta permanece");
    assert_eq!(arroz["amount"], 200.0, "la receta no se recalcula");
    let (_, source_recipe_id) = item_sources(&state, "Arroz").await;
    assert_eq!(source_recipe_id.as_deref(), Some(recipe.as_str()));

    // El plan se recalcula y el manual se conserva.
    assert!(
        find_item(&groups, "Tomate").is_some(),
        "el plan se consolida"
    );
    assert!(find_item(&groups, "Leche").is_some(), "el manual permanece");
}

/// Escenario: Mismo nombre con familias distintas — cada línea conserva la
/// receta que la aporta (la atribución es por nombre, familia y unidad canónica).
#[tokio::test]
async fn from_recipe_same_name_different_units_keeps_own_recipe() {
    let state = test_state().await;

    let recipe_a = create_recipe(
        &state,
        "Receta con leche",
        2,
        json!([{"name": "Leche", "amount": 500.0, "unit": "ml"}]),
    )
    .await;
    let recipe_b = create_recipe(
        &state,
        "Receta con leche en unidades",
        2,
        json!([{"name": "Leche", "amount": 1.0, "unit": "ud"}]),
    )
    .await;

    for recipe in [&recipe_a, &recipe_b] {
        let (status, _) = common::request(
            state.clone(),
            "POST",
            "/api/shopping-list/from-recipe",
            Some(json!({"recipe_id": recipe, "servings": 2})),
        )
        .await;
        assert_eq!(status, StatusCode::OK);
    }

    let lines: Vec<(String, String)> = sqlx::query_as(
        "SELECT unit, source_recipe_id FROM shopping_list_items \
         WHERE name = 'Leche' ORDER BY unit",
    )
    .fetch_all(&state.pool)
    .await
    .expect("leer líneas de leche");

    assert_eq!(lines.len(), 2, "dos líneas de leche: {lines:?}");
    let (_, ml_source) = lines
        .iter()
        .find(|(unit, _)| unit == "ml")
        .expect("línea en ml");
    let (_, ud_source) = lines
        .iter()
        .find(|(unit, _)| unit == "ud")
        .expect("línea en ud");
    assert_eq!(
        ml_source, &recipe_a,
        "la línea en ml proviene de la receta A"
    );
    assert_eq!(
        ud_source, &recipe_b,
        "la línea en ud proviene de la receta B"
    );
}

/// Escenario: Mitad de raciones — 4 → 2 raciones dividen la cantidad.
#[tokio::test]
async fn from_recipe_halves_servings() {
    let state = test_state().await;
    let recipe = create_recipe(
        &state,
        "Receta para cuatro",
        4,
        json!([{"name": "Arroz", "amount": 100.0, "unit": "g"}]),
    )
    .await;

    let (status, groups) = common::request(
        state.clone(),
        "POST",
        "/api/shopping-list/from-recipe",
        Some(json!({"recipe_id": recipe, "servings": 2})),
    )
    .await;
    assert_eq!(status, StatusCode::OK, "respuesta: {groups}");
    assert_eq!(
        find_item(&groups, "Arroz").expect("arroz")["amount"],
        50.0,
        "4 -> 2 raciones reduce a la mitad"
    );

    // Sin `servings` se usan las 4 raciones de la receta (factor 1).
    let (status, groups) = common::request(
        state,
        "POST",
        "/api/shopping-list/from-recipe",
        Some(json!({"recipe_id": recipe})),
    )
    .await;
    assert_eq!(status, StatusCode::OK, "respuesta: {groups}");
    assert_eq!(
        find_item(&groups, "Arroz").expect("arroz")["amount"],
        100.0,
        "sin servings se usan las 4 raciones de la receta"
    );
}

/// Escenario: El JSON de respuesta no filtra las columnas internas de procedencia.
#[tokio::test]
async fn from_recipe_response_hides_source_columns() {
    let state = test_state().await;
    let recipe = create_recipe(
        &state,
        "Receta base",
        2,
        json!([{"name": "Arroz", "amount": 100.0, "unit": "g"}]),
    )
    .await;

    let (status, groups) = common::request(
        state,
        "POST",
        "/api/shopping-list/from-recipe",
        Some(json!({"recipe_id": recipe, "servings": 2})),
    )
    .await;
    assert_eq!(status, StatusCode::OK, "respuesta: {groups}");

    for item in all_items(&groups) {
        let object = item.as_object().expect("ítem como objeto JSON");
        assert!(
            !object.contains_key("source_recipe_id"),
            "el JSON no debe exponer source_recipe_id: {item}"
        );
        assert!(
            !object.contains_key("source_plan_id"),
            "el JSON no debe exponer source_plan_id: {item}"
        );
    }
}

/// Escenario: `recipe_id` en blanco — 400 sin tocar la lista.
#[tokio::test]
async fn from_recipe_rejects_blank_recipe_id() {
    let state = test_state().await;
    add_manual_item(&state, "Leche", 1.0, "l").await;

    for recipe_id in ["", "   "] {
        let (status, _) = common::request(
            state.clone(),
            "POST",
            "/api/shopping-list/from-recipe",
            Some(json!({"recipe_id": recipe_id, "servings": 2})),
        )
        .await;
        assert_eq!(
            status,
            StatusCode::BAD_REQUEST,
            "recipe_id en blanco {recipe_id:?} debe dar 400"
        );
    }

    assert_eq!(
        recipe_derived_count(&state).await,
        0,
        "sin líneas de receta"
    );
    let (_, groups) = common::request(state, "GET", "/api/shopping-list", None).await;
    assert!(find_item(&groups, "Leche").is_some(), "el manual permanece");
}

/// Escenario: `recipe_id` con espacios — se recorta antes de buscar la receta.
#[tokio::test]
async fn from_recipe_trims_recipe_id_whitespace() {
    let state = test_state().await;
    let recipe = create_recipe(
        &state,
        "Receta base",
        2,
        json!([{"name": "Arroz", "amount": 100.0, "unit": "g"}]),
    )
    .await;

    let (status, groups) = common::request(
        state,
        "POST",
        "/api/shopping-list/from-recipe",
        Some(json!({"recipe_id": format!("  {recipe} "), "servings": 2})),
    )
    .await;
    assert_eq!(status, StatusCode::OK, "respuesta: {groups}");
    assert!(
        find_item(&groups, "Arroz").is_some(),
        "la receta se encuentra tras recortar el id"
    );
}

/// Escenario: Sin permiso — 401 sin sesión y 403 sin `write:shopping`.
#[tokio::test]
async fn from_recipe_without_credentials_returns_401() {
    let idp = MockIdp::start().await;
    let state = test_state_oidc(&idp.issuer, USER_GROUPS).await;

    let status = call(
        &state,
        "/api/shopping-list/from-recipe",
        None,
        Some(json!({"recipe_id": "x", "servings": 2})),
    )
    .await;
    assert_eq!(status, StatusCode::UNAUTHORIZED);
}

#[tokio::test]
async fn from_recipe_without_permission_returns_403() {
    let idp = MockIdp::start().await;
    let state = test_state_oidc(&idp.issuer, USER_GROUPS).await;

    let token = access_token(&idp.issuer, "openid profile read:shopping", API_AUDIENCE);
    let status = call(
        &state,
        "/api/shopping-list/from-recipe",
        Some(&token),
        Some(json!({"recipe_id": "x", "servings": 2})),
    )
    .await;
    assert_eq!(status, StatusCode::FORBIDDEN);

    assert_eq!(
        recipe_derived_count(&state).await,
        0,
        "no se modifica la lista sin permiso"
    );
}
