//! Tests de integración de los endpoints de la lista de la compra.

mod common;

use axum::http::StatusCode;
use oximeal_backend::error::AppError;
use serde_json::{Value, json};

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

/// Busca un ítem por nombre en la lista agrupada y devuelve su objeto.
fn find_item<'a>(groups: &'a Value, name: &str) -> Option<&'a Value> {
    groups.as_array()?.iter().find_map(|group| {
        group["items"]
            .as_array()?
            .iter()
            .find(|item| item["name"] == name)
    })
}

#[tokio::test]
async fn manual_create_uses_heuristic_category() {
    let state = common::test_state().await;

    let (status, body) = common::request(
        state.clone(),
        "POST",
        "/api/shopping-list",
        Some(json!({"name": "Tomate", "amount": 2.0, "unit": "ud"})),
    )
    .await;

    assert_eq!(status, StatusCode::CREATED);
    assert_eq!(body["category"], "Verdulería");
    assert_eq!(body["checked"], false);

    let (status, groups) = common::request(state, "GET", "/api/shopping-list", None).await;
    assert_eq!(status, StatusCode::OK);
    let item = find_item(&groups, "Tomate").expect("tomate en la lista");
    assert_eq!(item["category"], "Verdulería");
}

#[tokio::test]
async fn create_without_client_id_returns_201_and_null_client_id() {
    let state = common::test_state().await;

    let (status, body) = common::request(
        state,
        "POST",
        "/api/shopping-list",
        Some(json!({"name": "Tomate", "amount": 2.0, "unit": "ud"})),
    )
    .await;

    assert_eq!(status, StatusCode::CREATED);
    assert_eq!(body["client_id"], Value::Null);
}

#[tokio::test]
async fn create_with_new_client_id_returns_201_and_echoes_it() {
    let state = common::test_state().await;

    let (status, body) = common::request(
        state,
        "POST",
        "/api/shopping-list",
        Some(json!({"name": "Tomate", "amount": 2.0, "unit": "ud", "client_id": "abc"})),
    )
    .await;

    assert_eq!(status, StatusCode::CREATED);
    assert_eq!(body["client_id"], "abc");
}

#[tokio::test]
async fn create_twice_with_same_client_id_returns_existing_without_duplicating() {
    let state = common::test_state().await;

    let (first_status, first) = common::request(
        state.clone(),
        "POST",
        "/api/shopping-list",
        Some(json!({"name": "Tomate", "amount": 2.0, "unit": "ud", "client_id": "abc"})),
    )
    .await;
    assert_eq!(first_status, StatusCode::CREATED);

    let (status, second) = common::request(
        state.clone(),
        "POST",
        "/api/shopping-list",
        Some(json!({"name": "Tomate", "amount": 2.0, "unit": "ud", "client_id": "abc"})),
    )
    .await;

    assert_eq!(status, StatusCode::OK, "el alta repetida devuelve 200");
    assert_eq!(second["id"], first["id"], "devuelve el ítem existente");

    let count: i64 =
        sqlx::query_scalar("SELECT count(*) FROM shopping_list_items WHERE client_id = 'abc'")
            .fetch_one(&state.pool)
            .await
            .expect("contar ítems con client_id");
    assert_eq!(count, 1, "no debe duplicar el ítem");
}

#[tokio::test]
async fn concurrent_creates_with_same_client_id_create_a_single_row() {
    let state = common::test_state().await;

    let payload = json!({"name": "Tomate", "amount": 2.0, "unit": "ud", "client_id": "race"});
    let (first, second) = tokio::join!(
        common::request(
            state.clone(),
            "POST",
            "/api/shopping-list",
            Some(payload.clone())
        ),
        common::request(state.clone(), "POST", "/api/shopping-list", Some(payload)),
    );

    let statuses = [first.0, second.0];
    assert!(
        statuses.contains(&StatusCode::CREATED),
        "una de las altas debe crear el ítem: {statuses:?}"
    );
    assert!(
        statuses
            .iter()
            .all(|status| *status == StatusCode::CREATED || *status == StatusCode::OK),
        "ninguna alta debe fallar: {statuses:?}"
    );
    assert_eq!(
        first.1["id"], second.1["id"],
        "ambas respuestas apuntan al mismo ítem"
    );

    let count: i64 =
        sqlx::query_scalar("SELECT count(*) FROM shopping_list_items WHERE client_id = 'race'")
            .fetch_one(&state.pool)
            .await
            .expect("contar ítems con client_id");
    assert_eq!(
        count, 1,
        "nunca debe haber dos filas con el mismo client_id"
    );
}

#[tokio::test]
async fn list_exposes_client_id() {
    let state = common::test_state().await;

    let (status, _) = common::request(
        state.clone(),
        "POST",
        "/api/shopping-list",
        Some(json!({"name": "Leche", "amount": 1.0, "unit": "l", "client_id": "lista"})),
    )
    .await;
    assert_eq!(status, StatusCode::CREATED);

    let (status, groups) = common::request(state, "GET", "/api/shopping-list", None).await;
    assert_eq!(status, StatusCode::OK);
    let item = find_item(&groups, "Leche").expect("leche en la lista");
    assert_eq!(item["client_id"], "lista");
}

#[tokio::test]
async fn patch_toggles_checked() {
    let state = common::test_state().await;

    let (_, created) = common::request(
        state.clone(),
        "POST",
        "/api/shopping-list",
        Some(json!({"name": "Leche", "amount": 1.0, "unit": "l"})),
    )
    .await;
    let id = created["id"].as_str().expect("id de ítem");

    let (status, body) = common::request(
        state,
        "PATCH",
        &format!("/api/shopping-list/{id}"),
        Some(json!({"checked": true})),
    )
    .await;
    assert_eq!(status, StatusCode::OK);
    assert_eq!(body["checked"], true);
}

#[tokio::test]
async fn patch_with_empty_body_returns_400() {
    let state = common::test_state().await;

    let (_, created) = common::request(
        state.clone(),
        "POST",
        "/api/shopping-list",
        Some(json!({"name": "Leche", "amount": 1.0, "unit": "l"})),
    )
    .await;
    let id = created["id"].as_str().expect("id de ítem");

    let (status, _) = common::request(
        state,
        "PATCH",
        &format!("/api/shopping-list/{id}"),
        Some(json!({})),
    )
    .await;
    assert_eq!(status, StatusCode::BAD_REQUEST);
}

#[tokio::test]
async fn patch_updates_category() {
    let state = common::test_state().await;

    let (_, created) = common::request(
        state.clone(),
        "POST",
        "/api/shopping-list",
        Some(json!({"name": "Leche", "amount": 1.0, "unit": "l"})),
    )
    .await;
    let id = created["id"].as_str().expect("id de ítem");

    let (status, body) = common::request(
        state.clone(),
        "PATCH",
        &format!("/api/shopping-list/{id}"),
        Some(json!({"category": "Verdulería"})),
    )
    .await;
    assert_eq!(status, StatusCode::OK);
    assert_eq!(body["category"], "Verdulería");

    let (status, groups) = common::request(state, "GET", "/api/shopping-list", None).await;
    assert_eq!(status, StatusCode::OK);
    let item = find_item(&groups, "Leche").expect("leche en la lista");
    assert_eq!(item["category"], "Verdulería");
}

#[tokio::test]
async fn patch_rejects_unknown_category() {
    let state = common::test_state().await;

    let (_, created) = common::request(
        state.clone(),
        "POST",
        "/api/shopping-list",
        Some(json!({"name": "Leche", "amount": 1.0, "unit": "l"})),
    )
    .await;
    let id = created["id"].as_str().expect("id de ítem");
    let original = created["category"].as_str().expect("categoría inicial");

    let (status, body) = common::request(
        state.clone(),
        "PATCH",
        &format!("/api/shopping-list/{id}"),
        Some(json!({"category": "Frutas"})),
    )
    .await;
    assert_eq!(status, StatusCode::BAD_REQUEST);
    let message = body["message"].as_str().unwrap_or_default().to_lowercase();
    assert!(
        message.contains("category") || message.contains("categoría"),
        "el error debe mencionar la categoría, pero fue: {message:?}"
    );

    let (status, groups) = common::request(state, "GET", "/api/shopping-list", None).await;
    assert_eq!(status, StatusCode::OK);
    let item = find_item(&groups, "Leche").expect("leche en la lista");
    assert_eq!(item["category"], original);
}

#[tokio::test]
async fn patch_category_and_checked_together() {
    let state = common::test_state().await;

    let (_, created) = common::request(
        state.clone(),
        "POST",
        "/api/shopping-list",
        Some(json!({"name": "Leche", "amount": 1.0, "unit": "l"})),
    )
    .await;
    let id = created["id"].as_str().expect("id de ítem");

    let (status, body) = common::request(
        state,
        "PATCH",
        &format!("/api/shopping-list/{id}"),
        Some(json!({"category": "Verdulería", "checked": true})),
    )
    .await;
    assert_eq!(status, StatusCode::OK);
    assert_eq!(body["category"], "Verdulería");
    assert_eq!(body["checked"], true);
}

#[tokio::test]
async fn consolidate_sums_repeated_ingredients_and_keeps_manual_items() {
    let state = common::test_state().await;

    // Dos recetas con el mismo ingrediente: 2 + 3 = 5 ud de tomate.
    let r1 = create_recipe(&state, "Receta 1", "Tomate", 2.0, "ud").await;
    let r2 = create_recipe(&state, "Receta 2", "Tomate", 3.0, "ud").await;

    for payload in [
        json!({"date": "2026-03-02", "meal_type": "lunch", "recipe_id": r1, "role": "main"}),
        json!({"date": "2026-03-03", "meal_type": "dinner", "recipe_id": r2, "role": "main"}),
    ] {
        let (status, _) =
            common::request(state.clone(), "POST", "/api/meal-plan", Some(payload)).await;
        assert_eq!(status, StatusCode::CREATED);
    }

    // Ítem manual que NO debe borrarse al consolidar.
    let (status, _) = common::request(
        state.clone(),
        "POST",
        "/api/shopping-list",
        Some(json!({"name": "Leche", "amount": 1.0, "unit": "l"})),
    )
    .await;
    assert_eq!(status, StatusCode::CREATED);

    let (status, groups) = common::request(
        state,
        "POST",
        "/api/shopping-list/consolidate",
        Some(json!({"start_date": "2026-03-01", "end_date": "2026-03-07"})),
    )
    .await;
    assert_eq!(status, StatusCode::OK);

    let tomate = find_item(&groups, "Tomate").expect("tomate consolidado");
    assert_eq!(tomate["amount"], 5.0);
    assert_eq!(tomate["unit"], "ud");
    assert_eq!(tomate["category"], "Verdulería");

    // El ítem manual se conserva.
    assert!(
        find_item(&groups, "Leche").is_some(),
        "el ítem manual se conserva"
    );
}

#[tokio::test]
async fn consolidate_regenerates_plan_items_on_repeat() {
    let state = common::test_state().await;

    let r1 = create_recipe(&state, "Receta 1", "Tomate", 2.0, "ud").await;
    let (_, _) = common::request(
        state.clone(),
        "POST",
        "/api/meal-plan",
        Some(json!({"date": "2026-03-02", "meal_type": "lunch", "recipe_id": r1, "role": "main"})),
    )
    .await;

    // Consolidar dos veces no debe duplicar las cantidades.
    for _ in 0..2 {
        let (status, _) = common::request(
            state.clone(),
            "POST",
            "/api/shopping-list/consolidate",
            Some(json!({"start_date": "2026-03-01", "end_date": "2026-03-07"})),
        )
        .await;
        assert_eq!(status, StatusCode::OK);
    }

    let (_, groups) = common::request(state, "GET", "/api/shopping-list", None).await;
    let tomate = find_item(&groups, "Tomate").expect("tomate consolidado");
    assert_eq!(tomate["amount"], 2.0, "no debe duplicarse al reconsolidar");
}

#[tokio::test]
async fn delete_completed_leaves_only_unchecked() {
    let state = common::test_state().await;

    let (_, a) = common::request(
        state.clone(),
        "POST",
        "/api/shopping-list",
        Some(json!({"name": "Tomate", "amount": 1.0, "unit": "ud"})),
    )
    .await;
    let id_a = a["id"].as_str().expect("id a");

    let (_, b) = common::request(
        state.clone(),
        "POST",
        "/api/shopping-list",
        Some(json!({"name": "Leche", "amount": 1.0, "unit": "l"})),
    )
    .await;
    let id_b = b["id"].as_str().expect("id b");

    let (status, _) = common::request(
        state.clone(),
        "PATCH",
        &format!("/api/shopping-list/{id_a}"),
        Some(json!({"checked": true})),
    )
    .await;
    assert_eq!(status, StatusCode::OK);

    let (status, _) = common::request(
        state.clone(),
        "DELETE",
        "/api/shopping-list/completed",
        None,
    )
    .await;
    assert_eq!(status, StatusCode::NO_CONTENT);

    let (_, groups) = common::request(state, "GET", "/api/shopping-list", None).await;
    assert!(
        find_item(&groups, "Tomate").is_none(),
        "el marcado se elimina"
    );
    assert!(
        find_item(&groups, "Leche").is_some(),
        "el no marcado permanece"
    );
    // El id_b se usa para documentar que se conserva el ítem no marcado.
    assert!(!id_b.is_empty());
}

#[tokio::test]
async fn consolidate_scales_up_by_entry_servings() {
    let state = common::test_state().await;

    // Receta por defecto (servings = 2) con 100 g.
    let r1 = create_recipe(&state, "Receta escala", "Arroz", 100.0, "g").await;
    let (status, _) = common::request(
        state.clone(),
        "POST",
        "/api/meal-plan",
        Some(json!({
            "date": "2026-03-02", "meal_type": "lunch",
            "recipe_id": r1, "role": "main", "servings": 4
        })),
    )
    .await;
    assert_eq!(status, StatusCode::CREATED);

    let (status, groups) = common::request(
        state,
        "POST",
        "/api/shopping-list/consolidate",
        Some(json!({"start_date": "2026-03-01", "end_date": "2026-03-07"})),
    )
    .await;
    assert_eq!(status, StatusCode::OK);

    let arroz = find_item(&groups, "Arroz").expect("arroz consolidado");
    assert_eq!(
        arroz["amount"], 200.0,
        "planificada a 4 raciones duplica: 100 g -> 200 g"
    );
}

#[tokio::test]
async fn consolidate_scales_down_when_recipe_has_more_servings() {
    let state = common::test_state().await;

    let (status, body) = common::request(
        state.clone(),
        "POST",
        "/api/recipes",
        Some(json!({
            "name": "Receta 4 raciones", "category": "main", "servings": 4,
            "ingredients": [{"name": "Arroz", "amount": 100.0, "unit": "g"}]
        })),
    )
    .await;
    assert_eq!(status, StatusCode::CREATED);
    let r1 = body["id"].as_str().expect("id").to_string();

    let (status, _) = common::request(
        state.clone(),
        "POST",
        "/api/meal-plan",
        Some(json!({
            "date": "2026-03-02", "meal_type": "lunch",
            "recipe_id": r1, "role": "main", "servings": 2
        })),
    )
    .await;
    assert_eq!(status, StatusCode::CREATED);

    let (_, groups) = common::request(
        state,
        "POST",
        "/api/shopping-list/consolidate",
        Some(json!({"start_date": "2026-03-01", "end_date": "2026-03-07"})),
    )
    .await;

    let arroz = find_item(&groups, "Arroz").expect("arroz consolidado");
    assert_eq!(
        arroz["amount"], 50.0,
        "receta de 4 raciones planificada a 2 => mitad"
    );
}

#[tokio::test]
async fn consolidate_without_explicit_servings_keeps_amount() {
    let state = common::test_state().await;

    let r1 = create_recipe(&state, "Receta base", "Arroz", 100.0, "g").await;
    let (status, _) = common::request(
        state.clone(),
        "POST",
        "/api/meal-plan",
        Some(json!({"date": "2026-03-02", "meal_type": "lunch", "recipe_id": r1, "role": "main"})),
    )
    .await;
    assert_eq!(status, StatusCode::CREATED);

    let (_, groups) = common::request(
        state,
        "POST",
        "/api/shopping-list/consolidate",
        Some(json!({"start_date": "2026-03-01", "end_date": "2026-03-07"})),
    )
    .await;

    let arroz = find_item(&groups, "Arroz").expect("arroz consolidado");
    assert_eq!(
        arroz["amount"], 100.0,
        "sin raciones explícitas el factor es 1"
    );
}

#[tokio::test]
async fn consolidate_treats_null_servings_as_default_two() {
    let state = common::test_state().await;

    // Inserta con SQL directo una receta y una entrada con `servings = NULL`
    // (la API nunca persiste NULL: default 2), para ejercitar el COALESCE.
    sqlx::query(
        "INSERT INTO recipes (id, name, category, ingredients, servings) \
         VALUES ('rn', 'Receta NULL', 'main', ?, NULL)",
    )
    .bind(r#"[{"name":"Arroz","amount":100.0,"unit":"g"}]"#)
    .execute(&state.pool)
    .await
    .expect("insertar receta");

    sqlx::query(
        "INSERT INTO meal_plan_entries (id, date, meal_type, recipe_id, role, servings) \
         VALUES ('en', '2026-03-02', 'lunch', 'rn', 'main', NULL)",
    )
    .execute(&state.pool)
    .await
    .expect("insertar entrada");

    let (status, groups) = common::request(
        state,
        "POST",
        "/api/shopping-list/consolidate",
        Some(json!({"start_date": "2026-03-01", "end_date": "2026-03-07"})),
    )
    .await;
    assert_eq!(status, StatusCode::OK);

    let arroz = find_item(&groups, "Arroz").expect("arroz consolidado");
    assert_eq!(arroz["amount"], 100.0, "NULL/NULL => factor 1");
}

#[tokio::test]
async fn consolidate_sums_scaled_amounts_of_same_recipe() {
    let state = common::test_state().await;

    // Receta por defecto (servings = 2) con 100 g.
    let r1 = create_recipe(&state, "Receta sumada", "Arroz", 100.0, "g").await;

    for (date, servings) in [("2026-03-02", 4), ("2026-03-03", 1)] {
        let (status, _) = common::request(
            state.clone(),
            "POST",
            "/api/meal-plan",
            Some(json!({
                "date": date, "meal_type": "lunch",
                "recipe_id": r1, "role": "main", "servings": servings
            })),
        )
        .await;
        assert_eq!(status, StatusCode::CREATED);
    }

    let (_, groups) = common::request(
        state,
        "POST",
        "/api/shopping-list/consolidate",
        Some(json!({"start_date": "2026-03-01", "end_date": "2026-03-07"})),
    )
    .await;

    // 100 g x (4/2) = 200 g  +  100 g x (1/2) = 50 g  =>  250 g
    let arroz = find_item(&groups, "Arroz").expect("arroz consolidado");
    assert_eq!(
        arroz["amount"], 250.0,
        "cada entrada escala por su factor y se suman"
    );
}

// ---------------------------------------------------------------------------
// `POST /api/shopping-list/from-plan` — volcado aditivo desde el plan.
// ---------------------------------------------------------------------------

/// Añade una entrada de plan a la fecha indicada y comprueba el 201.
async fn add_plan_entry(
    state: &oximeal_backend::state::AppState,
    date: &str,
    recipe_id: &str,
    servings: i64,
) {
    let (status, body) = common::request(
        state.clone(),
        "POST",
        "/api/meal-plan",
        Some(json!({
            "date": date,
            "meal_type": "lunch",
            "recipe_id": recipe_id,
            "role": "main",
            "servings": servings,
        })),
    )
    .await;
    assert_eq!(status, StatusCode::CREATED, "crear entrada: {body}");
}

/// Crea una receta con un único ingrediente y raciones explícitas; devuelve su id.
///
/// Se usa cuando el factor de escalado debe ser 1 (raciones de la receta iguales
/// a las de la entrada) para aislar la consolidación de la aritmética de raciones.
async fn create_recipe_with_servings(
    state: &oximeal_backend::state::AppState,
    name: &str,
    ingredient_name: &str,
    amount: f64,
    unit: &str,
    servings: i64,
) -> String {
    let (status, body) = common::request(
        state.clone(),
        "POST",
        "/api/recipes",
        Some(json!({
            "name": name,
            "category": "main",
            "servings": servings,
            "ingredients": [{"name": ingredient_name, "amount": amount, "unit": unit}]
        })),
    )
    .await;
    assert_eq!(status, StatusCode::CREATED, "crear receta: {body}");
    body["id"].as_str().expect("id de receta").to_string()
}

/// Devuelve todas las líneas con ese nombre presentes en la lista agrupada.
fn find_all_items<'a>(groups: &'a Value, name: &str) -> Vec<&'a Value> {
    let mut found = Vec::new();
    if let Some(array) = groups.as_array() {
        for group in array {
            if let Some(items) = group["items"].as_array() {
                for item in items {
                    if item["name"] == name {
                        found.push(item);
                    }
                }
            }
        }
    }
    found
}

/// Escenario: volcado de un día. La lista se puebla con los ingredientes del día
/// y las cantidades se escalan por las raciones de la entrada y de la receta.
#[tokio::test]
async fn from_plan_adds_day_ingredients() {
    let state = common::test_state().await;

    // Receta de 4 raciones con 100 g; la entrada pide 2 raciones => 50 g.
    let (status, body) = common::request(
        state.clone(),
        "POST",
        "/api/recipes",
        Some(json!({
            "name": "Receta día",
            "category": "main",
            "servings": 4,
            "ingredients": [{"name": "Arroz", "amount": 100.0, "unit": "g"}]
        })),
    )
    .await;
    assert_eq!(status, StatusCode::CREATED);
    let recipe = body["id"].as_str().expect("id de receta").to_string();
    add_plan_entry(&state, "2026-03-02", &recipe, 2).await;

    let (status, groups) = common::request(
        state.clone(),
        "POST",
        "/api/shopping-list/from-plan",
        Some(json!({"start_date": "2026-03-02", "end_date": "2026-03-02"})),
    )
    .await;
    assert_eq!(status, StatusCode::OK, "respuesta: {groups}");

    let arroz = find_item(&groups, "Arroz").expect("arroz del día");
    assert_eq!(arroz["amount"], 50.0, "receta de 4 raciones a 2 => mitad");
    assert_eq!(arroz["unit"], "g");
    assert_eq!(arroz["category"], "Despensa");
}

/// Escenario: volcado de una semana. Los ingredientes repetidos se suman.
#[tokio::test]
async fn from_plan_adds_week_ingredients_and_sums_repeats() {
    let state = common::test_state().await;

    let recipe = create_recipe(&state, "Receta semana", "Tomate", 2.0, "ud").await;
    for date in [
        "2026-03-02",
        "2026-03-03",
        "2026-03-04",
        "2026-03-05",
        "2026-03-06",
        "2026-03-07",
        "2026-03-08",
    ] {
        add_plan_entry(&state, date, &recipe, 2).await;
    }

    let (status, groups) = common::request(
        state.clone(),
        "POST",
        "/api/shopping-list/from-plan",
        Some(json!({"start_date": "2026-03-02", "end_date": "2026-03-08"})),
    )
    .await;
    assert_eq!(status, StatusCode::OK, "respuesta: {groups}");

    let tomate = find_item(&groups, "Tomate").expect("tomate de la semana");
    assert_eq!(tomate["amount"], 14.0, "7 días x 2 ud");
}

/// Las líneas del plan quedan **separadas** de los ítems manuales: no se fusionan.
#[tokio::test]
async fn from_plan_keeps_plan_line_separate_from_manual() {
    let state = common::test_state().await;

    // Ítem manual: Tomate 2 ud.
    let (status, manual) = common::request(
        state.clone(),
        "POST",
        "/api/shopping-list",
        Some(json!({"name": "Tomate", "amount": 2.0, "unit": "ud"})),
    )
    .await;
    assert_eq!(status, StatusCode::CREATED);
    let manual_id = manual["id"].as_str().expect("id manual").to_string();

    // Plan: Tomate 4 ud.
    let recipe = create_recipe(&state, "Receta tomate", "Tomate", 4.0, "ud").await;
    add_plan_entry(&state, "2026-03-02", &recipe, 2).await;

    let (status, groups) = common::request(
        state.clone(),
        "POST",
        "/api/shopping-list/from-plan",
        Some(json!({"start_date": "2026-03-02", "end_date": "2026-03-02"})),
    )
    .await;
    assert_eq!(status, StatusCode::OK, "respuesta: {groups}");

    // Hay DOS líneas «Tomate»: la manual y la del plan, sin sumarse.
    let tomates = find_all_items(&groups, "Tomate");
    assert_eq!(tomates.len(), 2, "manual y plan deben ser líneas distintas");

    let manual_line = tomates
        .iter()
        .find(|item| item["id"] == manual_id)
        .expect("la línea manual conserva su id");
    assert_eq!(manual_line["amount"], 2.0, "la manual no se toca");

    let plan_line = tomates
        .iter()
        .find(|item| item["id"] != manual_id)
        .expect("existe una línea del plan separada");
    assert_eq!(plan_line["amount"], 4.0, "la línea del plan aporta 4 ud");

    let count: i64 =
        sqlx::query_scalar("SELECT count(*) FROM shopping_list_items WHERE name = 'Tomate'")
            .fetch_one(&state.pool)
            .await
            .expect("contar tomates");
    assert_eq!(count, 2, "dos filas: la manual y la del plan");

    // La manual sigue sin fuente de plan; la del plan queda marcada.
    let manual_source: Option<String> =
        sqlx::query_scalar("SELECT source_plan_id FROM shopping_list_items WHERE id = ?")
            .bind(&manual_id)
            .fetch_one(&state.pool)
            .await
            .expect("fuente de la manual");
    assert!(
        manual_source.is_none(),
        "la manual no es una línea del plan"
    );

    let plan_source: Option<String> = sqlx::query_scalar(
        "SELECT source_plan_id FROM shopping_list_items \
         WHERE name = 'Tomate' AND id <> ?",
    )
    .bind(&manual_id)
    .fetch_one(&state.pool)
    .await
    .expect("fuente de la línea del plan");
    assert!(
        plan_source.is_some(),
        "la línea del plan va marcada con source_plan_id"
    );
}

/// Dentro del plan, la consolidación usa nombre normalizado y unidad canónica:
/// `1 kg` + `500 g` = una única línea `1500 g` (sin ítems manuales).
#[tokio::test]
async fn from_plan_consolidates_canonical_units_within_plan() {
    let state = common::test_state().await;

    // Dos recetas con raciones iguales a las de sus entradas (factor 1).
    let r_kg =
        create_recipe_with_servings(&state, "Receta harina kg", "Harina", 1.0, "kg", 2).await;
    let r_g = create_recipe_with_servings(&state, "Receta harina g", "Harina", 500.0, "g", 2).await;
    add_plan_entry(&state, "2026-03-02", &r_kg, 2).await;
    add_plan_entry(&state, "2026-03-03", &r_g, 2).await;

    let (status, groups) = common::request(
        state.clone(),
        "POST",
        "/api/shopping-list/from-plan",
        Some(json!({"start_date": "2026-03-02", "end_date": "2026-03-03"})),
    )
    .await;
    assert_eq!(status, StatusCode::OK, "respuesta: {groups}");

    let harinas = find_all_items(&groups, "Harina");
    assert_eq!(harinas.len(), 1, "una sola línea de harina");
    assert_eq!(harinas[0]["amount"], 1500.0, "1 kg + 500 g");
    assert_eq!(harinas[0]["unit"], "g");

    let count: i64 = sqlx::query_scalar(
        "SELECT count(*) FROM shopping_list_items \
         WHERE name = 'Harina' AND source_plan_id IS NOT NULL",
    )
    .fetch_one(&state.pool)
    .await
    .expect("contar harina del plan");
    assert_eq!(count, 1, "una única línea del plan");
}

/// Escenario: conservación de ítems manuales y volcados de receta.
#[tokio::test]
async fn from_plan_preserves_manual_and_recipe_items() {
    let state = common::test_state().await;

    // Volcado de receta: Leche 1 l.
    let recipe = create_recipe(&state, "Receta leche", "Leche", 1.0, "l").await;
    let (status, _) = common::request(
        state.clone(),
        "POST",
        "/api/shopping-list/from-recipe",
        Some(json!({"recipe_id": recipe})),
    )
    .await;
    assert_eq!(status, StatusCode::OK);

    // Ítem manual: Tomate 2 ud con client_id y marcado.
    let (status, manual) = common::request(
        state.clone(),
        "POST",
        "/api/shopping-list",
        Some(json!({"name": "Tomate", "amount": 2.0, "unit": "ud", "client_id": "abc"})),
    )
    .await;
    assert_eq!(status, StatusCode::CREATED);
    let manual_id = manual["id"].as_str().expect("id manual").to_string();
    let (status, _) = common::request(
        state.clone(),
        "PATCH",
        &format!("/api/shopping-list/{manual_id}"),
        Some(json!({"checked": true})),
    )
    .await;
    assert_eq!(status, StatusCode::OK);

    // Plan: Tomate 4 ud (línea propia, separada del manual).
    let recipe_plan = create_recipe(&state, "Receta tomate", "Tomate", 4.0, "ud").await;
    add_plan_entry(&state, "2026-03-02", &recipe_plan, 2).await;

    let (status, groups) = common::request(
        state.clone(),
        "POST",
        "/api/shopping-list/from-plan",
        Some(json!({"start_date": "2026-03-02", "end_date": "2026-03-02"})),
    )
    .await;
    assert_eq!(status, StatusCode::OK, "respuesta: {groups}");

    // El volcado de receta sigue ahí (canonicalizado a ml).
    let leche = find_item(&groups, "Leche").expect("leche conservada");
    assert_eq!(leche["amount"], 1000.0);
    assert_eq!(leche["unit"], "ml");

    // Hay dos líneas «Tomate»: la manual (intacta) y la del plan.
    let tomates = find_all_items(&groups, "Tomate");
    assert_eq!(tomates.len(), 2, "manual y plan separados");
    let manual_line = tomates
        .iter()
        .find(|item| item["id"] == manual_id)
        .expect("la línea manual conserva su id");
    assert_eq!(manual_line["amount"], 2.0);
    assert_eq!(manual_line["checked"], true);
    assert_eq!(manual_line["client_id"], "abc");

    let plan_line = tomates
        .iter()
        .find(|item| item["id"] != manual_id)
        .expect("existe una línea del plan separada");
    assert_eq!(plan_line["amount"], 4.0);

    // El volcado de receta conserva su fuente.
    let (_, source_recipe_id) = sqlx::query_as::<_, (Option<String>, Option<String>)>(
        "SELECT source_plan_id, source_recipe_id FROM shopping_list_items WHERE name = 'Leche'",
    )
    .fetch_one(&state.pool)
    .await
    .expect("leer fuente de leche");
    assert_eq!(source_recipe_id.as_deref(), Some(recipe.as_str()));
}

/// Escenario: ambos extremos son obligatorios.
#[tokio::test]
async fn from_plan_requires_both_dates() {
    let state = common::test_state().await;

    for body in [
        json!({}),
        json!({"start_date": "2026-03-02"}),
        json!({"end_date": "2026-03-02"}),
    ] {
        let (status, _) = common::request(
            state.clone(),
            "POST",
            "/api/shopping-list/from-plan",
            Some(body),
        )
        .await;
        assert_eq!(
            status,
            StatusCode::BAD_REQUEST,
            "cuerpo incompleto debe dar 400"
        );
    }
}

/// Escenario: rango inválido. Devuelve 400 y no modifica la lista.
#[tokio::test]
async fn from_plan_rejects_invalid_range() {
    let state = common::test_state().await;

    let (status, _) = common::request(
        state.clone(),
        "POST",
        "/api/shopping-list",
        Some(json!({"name": "Leche", "amount": 1.0, "unit": "l"})),
    )
    .await;
    assert_eq!(status, StatusCode::CREATED);

    let (status, body) = common::request(
        state.clone(),
        "POST",
        "/api/shopping-list/from-plan",
        Some(json!({"start_date": "2026-03-08", "end_date": "2026-03-02"})),
    )
    .await;
    assert_eq!(status, StatusCode::BAD_REQUEST);
    let message = body["message"].as_str().unwrap_or_default().to_lowercase();
    assert!(
        message.contains("start_date") || message.contains("end_date"),
        "el error debe mencionar el rango, pero fue: {message:?}"
    );

    // La lista no se modifica.
    let (_, groups) = common::request(state, "GET", "/api/shopping-list", None).await;
    assert!(find_item(&groups, "Leche").is_some(), "la lista no cambia");
}

/// Escenario: plan vacío. Devuelve 200 con la lista sin cambios.
#[tokio::test]
async fn from_plan_empty_plan_returns_unchanged_list() {
    let state = common::test_state().await;

    let (status, _) = common::request(
        state.clone(),
        "POST",
        "/api/shopping-list",
        Some(json!({"name": "Leche", "amount": 1.0, "unit": "l"})),
    )
    .await;
    assert_eq!(status, StatusCode::CREATED);

    let (status, groups) = common::request(
        state.clone(),
        "POST",
        "/api/shopping-list/from-plan",
        Some(json!({"start_date": "2026-03-02", "end_date": "2026-03-08"})),
    )
    .await;
    assert_eq!(status, StatusCode::OK, "respuesta: {groups}");

    let leche = find_item(&groups, "Leche").expect("leche conservada");
    assert_eq!(leche["amount"], 1.0);
    let count: i64 = sqlx::query_scalar("SELECT count(*) FROM shopping_list_items")
        .fetch_one(&state.pool)
        .await
        .expect("contar ítems");
    assert_eq!(count, 1, "no se añade nada");
}

/// El repositorio también valida el rango (defensa en profundidad).
#[tokio::test]
async fn from_plan_repo_rejects_invalid_range() {
    let state = common::test_state().await;

    let error =
        oximeal_backend::repo::shopping_list::from_plan(&state.pool, "2026-03-08", "2026-03-02")
            .await
            .expect_err("rango inválido debe fallar");

    assert!(matches!(error, AppError::Validation(_)));
}

/// La consolidación del plan redondea las cantidades a 2 decimales.
#[tokio::test]
async fn from_plan_rounds_accumulated_floats_to_two_decimals() {
    let state = common::test_state().await;

    // Dos entradas «Sal» en dos fechas del rango, con factor 1.
    let r1 = create_recipe_with_servings(&state, "Receta sal 1", "Sal", 0.1, "g", 2).await;
    let r2 = create_recipe_with_servings(&state, "Receta sal 2", "Sal", 0.2, "g", 2).await;
    add_plan_entry(&state, "2026-03-02", &r1, 2).await;
    add_plan_entry(&state, "2026-03-03", &r2, 2).await;

    let (status, groups) = common::request(
        state.clone(),
        "POST",
        "/api/shopping-list/from-plan",
        Some(json!({"start_date": "2026-03-02", "end_date": "2026-03-03"})),
    )
    .await;
    assert_eq!(status, StatusCode::OK, "respuesta: {groups}");

    let sales = find_all_items(&groups, "Sal");
    assert_eq!(sales.len(), 1, "una única línea de sal");
    assert_eq!(sales[0]["amount"], 0.3, "0.1 + 0.2 debe redondear a 0.3");

    let count: i64 = sqlx::query_scalar(
        "SELECT count(*) FROM shopping_list_items \
         WHERE name = 'Sal' AND source_plan_id IS NOT NULL",
    )
    .fetch_one(&state.pool)
    .await
    .expect("contar sal del plan");
    assert_eq!(count, 1, "una única línea del plan");
}

/// La consolidación del plan normaliza la unidad `Unknown`: `Cajas` y `cajas` coinciden.
#[tokio::test]
async fn from_plan_merges_unknown_units_case_insensitively() {
    let state = common::test_state().await;

    let r1 =
        create_recipe_with_servings(&state, "Receta huevos 1", "Huevos", 2.0, "cajas", 2).await;
    let r2 =
        create_recipe_with_servings(&state, "Receta huevos 2", "Huevos", 3.0, "Cajas", 2).await;
    add_plan_entry(&state, "2026-03-02", &r1, 2).await;
    add_plan_entry(&state, "2026-03-03", &r2, 2).await;

    let (status, groups) = common::request(
        state.clone(),
        "POST",
        "/api/shopping-list/from-plan",
        Some(json!({"start_date": "2026-03-02", "end_date": "2026-03-03"})),
    )
    .await;
    assert_eq!(status, StatusCode::OK, "respuesta: {groups}");

    let huevos = find_item(&groups, "Huevos").expect("huevos del plan");
    assert_eq!(huevos["amount"], 5.0, "2 cajas + 3 cajas");
    let count: i64 = sqlx::query_scalar(
        "SELECT count(*) FROM shopping_list_items \
         WHERE name = 'Huevos' AND source_plan_id IS NOT NULL",
    )
    .fetch_one(&state.pool)
    .await
    .expect("contar huevos del plan");
    assert_eq!(
        count, 1,
        "una sola línea del plan, sin duplicar por mayúsculas"
    );
}

/// Un ítem manual «al gusto» (0.0) no se toca; el aporte del plan va en otra línea.
#[tokio::test]
async fn from_plan_does_not_touch_unchanged_to_taste_item() {
    let state = common::test_state().await;

    let (status, body) = common::request(
        state.clone(),
        "POST",
        "/api/shopping-list",
        Some(json!({"name": "Sal", "amount": 0.0, "unit": "al gusto"})),
    )
    .await;
    assert_eq!(status, StatusCode::CREATED);
    let id = body["id"].as_str().expect("id de sal").to_string();

    sqlx::query("UPDATE shopping_list_items SET updated_at = '2020-01-01 00:00:00' WHERE id = ?")
        .bind(&id)
        .execute(&state.pool)
        .await
        .expect("fijar updated_at");

    let recipe = create_recipe(&state, "Receta sal", "Sal", 0.0, "al gusto").await;
    add_plan_entry(&state, "2026-03-02", &recipe, 2).await;

    let (status, groups) = common::request(
        state.clone(),
        "POST",
        "/api/shopping-list/from-plan",
        Some(json!({"start_date": "2026-03-02", "end_date": "2026-03-02"})),
    )
    .await;
    assert_eq!(status, StatusCode::OK, "respuesta: {groups}");

    let updated_at: String =
        sqlx::query_scalar("SELECT CAST(updated_at AS TEXT) FROM shopping_list_items WHERE id = ?")
            .bind(&id)
            .fetch_one(&state.pool)
            .await
            .expect("leer updated_at");
    assert_eq!(
        updated_at, "2020-01-01 00:00:00",
        "sin cantidad real no debe refrescarse updated_at"
    );

    // El ítem manual sigue sin fuente de plan.
    let manual_source: Option<String> =
        sqlx::query_scalar("SELECT source_plan_id FROM shopping_list_items WHERE id = ?")
            .bind(&id)
            .fetch_one(&state.pool)
            .await
            .expect("fuente de la manual");
    assert!(
        manual_source.is_none(),
        "la manual no es una línea del plan"
    );

    // El aporte del plan queda en una línea separada.
    let plan_count: i64 = sqlx::query_scalar(
        "SELECT count(*) FROM shopping_list_items \
         WHERE name = 'Sal' AND source_plan_id IS NOT NULL",
    )
    .fetch_one(&state.pool)
    .await
    .expect("contar sal del plan");
    assert_eq!(
        plan_count, 1,
        "la línea del plan es independiente de la manual"
    );
}

/// Repetir el mismo rango de `from-plan` no duplica la línea del plan.
#[tokio::test]
async fn from_plan_is_idempotent_on_repeat() {
    let state = common::test_state().await;

    let recipe = create_recipe(&state, "Receta tomate", "Tomate", 4.0, "ud").await;
    add_plan_entry(&state, "2026-03-02", &recipe, 2).await;

    for _ in 0..2 {
        let (status, _) = common::request(
            state.clone(),
            "POST",
            "/api/shopping-list/from-plan",
            Some(json!({"start_date": "2026-03-02", "end_date": "2026-03-02"})),
        )
        .await;
        assert_eq!(status, StatusCode::OK);
    }

    let (_, groups) = common::request(state.clone(), "GET", "/api/shopping-list", None).await;
    let tomates = find_all_items(&groups, "Tomate");
    assert_eq!(tomates.len(), 1, "una sola línea del plan");
    assert_eq!(tomates[0]["amount"], 4.0, "no debe acumularse al repetir");

    let count: i64 = sqlx::query_scalar(
        "SELECT count(*) FROM shopping_list_items \
         WHERE name = 'Tomate' AND source_plan_id IS NOT NULL",
    )
    .fetch_one(&state.pool)
    .await
    .expect("contar tomate del plan");
    assert_eq!(count, 1, "una única línea del plan");
}

/// Volcar rangos distintos acumula las entradas sin duplicar las coincidentes.
#[tokio::test]
async fn from_plan_accumulates_distinct_ranges() {
    let state = common::test_state().await;

    let r_tomate = create_recipe(&state, "Receta tomate", "Tomate", 2.0, "ud").await;
    let r_leche = create_recipe(&state, "Receta leche", "Leche", 1.0, "l").await;
    add_plan_entry(&state, "2026-03-02", &r_tomate, 2).await; // lunes
    add_plan_entry(&state, "2026-03-03", &r_leche, 2).await; // martes

    let (status, _) = common::request(
        state.clone(),
        "POST",
        "/api/shopping-list/from-plan",
        Some(json!({"start_date": "2026-03-02", "end_date": "2026-03-02"})),
    )
    .await;
    assert_eq!(status, StatusCode::OK);

    let (status, groups) = common::request(
        state.clone(),
        "POST",
        "/api/shopping-list/from-plan",
        Some(json!({"start_date": "2026-03-03", "end_date": "2026-03-03"})),
    )
    .await;
    assert_eq!(status, StatusCode::OK, "respuesta: {groups}");

    assert_eq!(
        find_all_items(&groups, "Tomate").len(),
        1,
        "tomate del lunes"
    );
    assert_eq!(
        find_all_items(&groups, "Leche").len(),
        1,
        "leche del martes"
    );

    let plan_lines: i64 = sqlx::query_scalar(
        "SELECT count(*) FROM shopping_list_items WHERE source_plan_id IS NOT NULL",
    )
    .fetch_one(&state.pool)
    .await
    .expect("contar líneas del plan");
    assert_eq!(
        plan_lines, 2,
        "ambas entradas quedan registradas como fuentes"
    );
}

/// Editar las raciones de una entrada y re-volcar reescala sin acumular.
#[tokio::test]
async fn from_plan_rescales_when_plan_servings_change() {
    let state = common::test_state().await;

    // Receta de 4 raciones con 100 g; entrada a 2 raciones => 50 g.
    let recipe =
        create_recipe_with_servings(&state, "Receta 4 raciones", "Arroz", 100.0, "g", 4).await;
    add_plan_entry(&state, "2026-03-02", &recipe, 2).await;

    let (status, groups) = common::request(
        state.clone(),
        "POST",
        "/api/shopping-list/from-plan",
        Some(json!({"start_date": "2026-03-02", "end_date": "2026-03-02"})),
    )
    .await;
    assert_eq!(status, StatusCode::OK, "respuesta: {groups}");
    let arroz = find_item(&groups, "Arroz").expect("arroz del plan");
    assert_eq!(arroz["amount"], 50.0, "100 g x (2/4)");

    // Recupera el id de la entrada y sube las raciones a 4.
    let (status, entries) = common::request(
        state.clone(),
        "GET",
        "/api/meal-plan?start_date=2026-03-02&end_date=2026-03-02",
        None,
    )
    .await;
    assert_eq!(status, StatusCode::OK);
    let entry_id = entries[0]["id"]
        .as_str()
        .expect("id de entrada")
        .to_string();
    let (status, _) = common::request(
        state.clone(),
        "PATCH",
        &format!("/api/meal-plan/{entry_id}"),
        Some(json!({"servings": 4})),
    )
    .await;
    assert_eq!(status, StatusCode::OK);

    // Re-volcar el MISMO rango reescala a 100 g (no 150 g acumulados).
    let (status, groups) = common::request(
        state.clone(),
        "POST",
        "/api/shopping-list/from-plan",
        Some(json!({"start_date": "2026-03-02", "end_date": "2026-03-02"})),
    )
    .await;
    assert_eq!(status, StatusCode::OK, "respuesta: {groups}");
    let arroz = find_item(&groups, "Arroz").expect("arroz reescalado");
    assert_eq!(arroz["amount"], 100.0, "100 g x (4/4), sin acumular");
}

/// Un rango que solapa con otro ya volcado no duplica las entradas (día ⊂ semana).
#[tokio::test]
async fn from_plan_deduplicates_overlapping_ranges() {
    let state = common::test_state().await;

    let recipe = create_recipe(&state, "Receta tomate", "Tomate", 2.0, "ud").await;
    add_plan_entry(&state, "2026-03-02", &recipe, 2).await; // lunes
    add_plan_entry(&state, "2026-03-03", &recipe, 2).await; // martes

    // Volcado del lunes.
    let (status, _) = common::request(
        state.clone(),
        "POST",
        "/api/shopping-list/from-plan",
        Some(json!({"start_date": "2026-03-02", "end_date": "2026-03-02"})),
    )
    .await;
    assert_eq!(status, StatusCode::OK);

    // Volcado de la semana completa (incluye el lunes ya volcado).
    let (status, groups) = common::request(
        state.clone(),
        "POST",
        "/api/shopping-list/from-plan",
        Some(json!({"start_date": "2026-03-02", "end_date": "2026-03-08"})),
    )
    .await;
    assert_eq!(status, StatusCode::OK, "respuesta: {groups}");

    let tomates = find_all_items(&groups, "Tomate");
    assert_eq!(tomates.len(), 1, "una sola línea del plan");
    assert_eq!(
        tomates[0]["amount"], 4.0,
        "lunes 2 + martes 2, sin duplicar"
    );
}

/// `consolidate` fija las fuentes al rango indicado: desaparece lo volcado antes.
#[tokio::test]
async fn consolidate_resets_plan_sources_to_range() {
    let state = common::test_state().await;

    let r_tomate = create_recipe(&state, "Receta tomate", "Tomate", 2.0, "ud").await;
    let r_leche = create_recipe(&state, "Receta leche", "Leche", 1.0, "l").await;
    add_plan_entry(&state, "2026-03-02", &r_tomate, 2).await; // lunes
    add_plan_entry(&state, "2026-03-04", &r_leche, 2).await; // miércoles

    // Volcado inicial del lunes: aparece el tomate.
    let (status, groups) = common::request(
        state.clone(),
        "POST",
        "/api/shopping-list/from-plan",
        Some(json!({"start_date": "2026-03-02", "end_date": "2026-03-02"})),
    )
    .await;
    assert_eq!(status, StatusCode::OK, "respuesta: {groups}");
    assert!(
        find_item(&groups, "Tomate").is_some(),
        "el lunes debe aparecer tras from-plan"
    );

    // Consolidar SOLO el miércoles fija las fuentes a ese rango.
    let (status, groups) = common::request(
        state.clone(),
        "POST",
        "/api/shopping-list/consolidate",
        Some(json!({"start_date": "2026-03-04", "end_date": "2026-03-04"})),
    )
    .await;
    assert_eq!(status, StatusCode::OK, "respuesta: {groups}");

    assert!(
        find_item(&groups, "Tomate").is_none(),
        "el volcado del lunes desaparece al consolidar otro rango"
    );
    assert!(
        find_item(&groups, "Leche").is_some(),
        "aparece la leche del rango consolidado"
    );

    let plan_lines: i64 = sqlx::query_scalar(
        "SELECT count(*) FROM shopping_list_items WHERE source_plan_id IS NOT NULL",
    )
    .fetch_one(&state.pool)
    .await
    .expect("contar líneas del plan");
    assert_eq!(plan_lines, 1, "solo las fuentes del rango consolidado");
}

/// Repetir `from-plan` recrea la línea del plan y resetea su `checked`.
///
/// Es el **mismo contrato** que `from-recipe`: las líneas derivadas se borran y
/// recrean siempre con `checked = 0`, de modo que repetir el volcado resetea el
/// estado "ya comprado". Los ítems manuales, en cambio, conservan su `checked`.
#[tokio::test]
async fn from_plan_rerun_resets_plan_line_checked() {
    let state = common::test_state().await;

    let recipe = create_recipe(&state, "Receta tomate", "Tomate", 4.0, "ud").await;
    add_plan_entry(&state, "2026-03-02", &recipe, 2).await;

    // Primer volcado: la línea del plan nace sin marcar.
    let (status, groups) = common::request(
        state.clone(),
        "POST",
        "/api/shopping-list/from-plan",
        Some(json!({"start_date": "2026-03-02", "end_date": "2026-03-02"})),
    )
    .await;
    assert_eq!(status, StatusCode::OK, "respuesta: {groups}");
    let tomate = find_item(&groups, "Tomate").expect("tomate del plan");
    assert_eq!(tomate["checked"], false);
    let plan_id = tomate["id"].as_str().expect("id del plan").to_string();

    // El usuario la marca como comprada.
    let (status, _) = common::request(
        state.clone(),
        "PATCH",
        &format!("/api/shopping-list/{plan_id}"),
        Some(json!({"checked": true})),
    )
    .await;
    assert_eq!(status, StatusCode::OK);

    // Repetir el MISMO `from-plan` recrea la línea: vuelve a `checked = false`.
    let (status, groups) = common::request(
        state.clone(),
        "POST",
        "/api/shopping-list/from-plan",
        Some(json!({"start_date": "2026-03-02", "end_date": "2026-03-02"})),
    )
    .await;
    assert_eq!(status, StatusCode::OK, "respuesta: {groups}");
    let tomate = find_item(&groups, "Tomate").expect("tomate re-volcado");
    assert_eq!(
        tomate["checked"], false,
        "las líneas del plan se recrean sin marcar (mismo contrato que from-recipe)"
    );
    assert_eq!(tomate["amount"], 4.0, "no se acumula al repetir");
}

/// Borrar una entrada del plan elimina su fuente (`ON DELETE CASCADE`); al
/// re-volcar, la línea del plan desaparece.
#[tokio::test]
async fn from_plan_drops_source_when_entry_deleted() {
    let state = common::test_state().await;

    let recipe = create_recipe(&state, "Receta tomate", "Tomate", 4.0, "ud").await;
    add_plan_entry(&state, "2026-03-02", &recipe, 2).await;

    let (status, groups) = common::request(
        state.clone(),
        "POST",
        "/api/shopping-list/from-plan",
        Some(json!({"start_date": "2026-03-02", "end_date": "2026-03-02"})),
    )
    .await;
    assert_eq!(status, StatusCode::OK, "respuesta: {groups}");
    let tomate = find_item(&groups, "Tomate").expect("tomate del plan");
    assert_eq!(tomate["amount"], 4.0);

    // Recupera el id de la entrada y bórrala.
    let (status, entries) = common::request(
        state.clone(),
        "GET",
        "/api/meal-plan?start_date=2026-03-02&end_date=2026-03-02",
        None,
    )
    .await;
    assert_eq!(status, StatusCode::OK);
    let entry_id = entries[0]["id"]
        .as_str()
        .expect("id de entrada")
        .to_string();
    let (status, _) = common::request(
        state.clone(),
        "DELETE",
        &format!("/api/meal-plan/{entry_id}"),
        None,
    )
    .await;
    assert_eq!(status, StatusCode::NO_CONTENT);

    // La FK `ON DELETE CASCADE` elimina la fuente; al reconstruir cae la línea.
    let (status, groups) = common::request(
        state.clone(),
        "POST",
        "/api/shopping-list/from-plan",
        Some(json!({"start_date": "2026-03-02", "end_date": "2026-03-02"})),
    )
    .await;
    assert_eq!(status, StatusCode::OK, "respuesta: {groups}");
    assert!(
        find_item(&groups, "Tomate").is_none(),
        "la línea del plan desaparece al borrar su entrada"
    );

    let plan_lines: i64 = sqlx::query_scalar(
        "SELECT count(*) FROM shopping_list_items WHERE source_plan_id IS NOT NULL",
    )
    .fetch_one(&state.pool)
    .await
    .expect("contar líneas del plan");
    assert_eq!(plan_lines, 0, "no quedan líneas del plan");
}

/// `from-recipe` no toca las líneas del plan: coexisten ambas.
#[tokio::test]
async fn from_recipe_after_from_plan_keeps_plan_lines() {
    let state = common::test_state().await;

    // Plan: Tomate 4 ud.
    let plan_recipe = create_recipe(&state, "Receta tomate", "Tomate", 4.0, "ud").await;
    add_plan_entry(&state, "2026-03-02", &plan_recipe, 2).await;

    let (status, _) = common::request(
        state.clone(),
        "POST",
        "/api/shopping-list/from-plan",
        Some(json!({"start_date": "2026-03-02", "end_date": "2026-03-02"})),
    )
    .await;
    assert_eq!(status, StatusCode::OK);

    // Volcado de otra receta: Leche 1 l.
    let recipe = create_recipe(&state, "Receta leche", "Leche", 1.0, "l").await;
    let (status, groups) = common::request(
        state.clone(),
        "POST",
        "/api/shopping-list/from-recipe",
        Some(json!({"recipe_id": recipe})),
    )
    .await;
    assert_eq!(status, StatusCode::OK, "respuesta: {groups}");

    let tomate = find_item(&groups, "Tomate").expect("la línea del plan se conserva");
    assert_eq!(tomate["amount"], 4.0);
    assert!(
        find_item(&groups, "Leche").is_some(),
        "la línea de receta se añade"
    );
}

/// `consolidate` fija las fuentes del **plan**, pero no toca las líneas
/// derivadas de recetas: conservan su `source_recipe_id`.
#[tokio::test]
async fn consolidate_preserves_recipe_lines() {
    let state = common::test_state().await;

    // Volcado de receta: Leche 1 l (canonicalizada a ml).
    let recipe = create_recipe(&state, "Receta leche", "Leche", 1.0, "l").await;
    let (status, _) = common::request(
        state.clone(),
        "POST",
        "/api/shopping-list/from-recipe",
        Some(json!({"recipe_id": recipe})),
    )
    .await;
    assert_eq!(status, StatusCode::OK);

    // Consolidar un rango distinto (sin entradas de plan) no debe tocarla.
    let (status, groups) = common::request(
        state.clone(),
        "POST",
        "/api/shopping-list/consolidate",
        Some(json!({"start_date": "2026-04-01", "end_date": "2026-04-07"})),
    )
    .await;
    assert_eq!(status, StatusCode::OK, "respuesta: {groups}");

    let leche = find_item(&groups, "Leche").expect("la línea de receta se conserva");
    assert_eq!(leche["amount"], 1000.0);
    assert_eq!(leche["unit"], "ml");

    let source_recipe_id: Option<String> =
        sqlx::query_scalar("SELECT source_recipe_id FROM shopping_list_items WHERE name = 'Leche'")
            .fetch_one(&state.pool)
            .await
            .expect("fuente de la línea de receta");
    assert_eq!(
        source_recipe_id.as_deref(),
        Some(recipe.as_str()),
        "la línea de receta conserva su procedencia"
    );
}
