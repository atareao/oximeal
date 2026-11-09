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

/// Escenario: fusión con un ítem manual existente. Suma la cantidad sin duplicar.
#[tokio::test]
async fn from_plan_merges_with_existing_manual_item() {
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

    let tomate = find_item(&groups, "Tomate").expect("tomate fusionado");
    assert_eq!(tomate["amount"], 6.0, "2 + 4");
    assert_eq!(tomate["id"], manual_id, "no se crea un duplicado");

    let count: i64 =
        sqlx::query_scalar("SELECT count(*) FROM shopping_list_items WHERE name = 'Tomate'")
            .fetch_one(&state.pool)
            .await
            .expect("contar tomates");
    assert_eq!(count, 1, "no debe haber duplicados");
}

/// La fusión usa nombre normalizado y unidad canónica: `1 kg` + `500 g` = `1500 g`.
#[tokio::test]
async fn from_plan_merges_using_canonical_unit() {
    let state = common::test_state().await;

    // Ítem manual: Harina 1 kg.
    let (status, _) = common::request(
        state.clone(),
        "POST",
        "/api/shopping-list",
        Some(json!({"name": "Harina", "amount": 1.0, "unit": "kg"})),
    )
    .await;
    assert_eq!(status, StatusCode::CREATED);

    // Plan: Harina 500 g.
    let recipe = create_recipe(&state, "Receta harina", "Harina", 500.0, "g").await;
    add_plan_entry(&state, "2026-03-02", &recipe, 2).await;

    let (status, groups) = common::request(
        state.clone(),
        "POST",
        "/api/shopping-list/from-plan",
        Some(json!({"start_date": "2026-03-02", "end_date": "2026-03-02"})),
    )
    .await;
    assert_eq!(status, StatusCode::OK, "respuesta: {groups}");

    let harina = find_item(&groups, "Harina").expect("harina fusionada");
    assert_eq!(harina["amount"], 1500.0, "1 kg + 500 g");
    assert_eq!(harina["unit"], "g");
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

    // Plan: Tomate 4 ud (se fusiona con el manual).
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

    // El manual se conserva, con su id, checked y client_id.
    let tomate = find_item(&groups, "Tomate").expect("tomate conservado");
    assert_eq!(tomate["id"], manual_id);
    assert_eq!(tomate["amount"], 6.0);
    assert_eq!(tomate["checked"], true);
    assert_eq!(tomate["client_id"], "abc");

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

/// La suma de cantidades se redondea a 2 decimales como el motor `consolidate`.
#[tokio::test]
async fn from_plan_rounds_accumulated_floats_to_two_decimals() {
    let state = common::test_state().await;

    let (status, _) = common::request(
        state.clone(),
        "POST",
        "/api/shopping-list",
        Some(json!({"name": "Sal", "amount": 0.1, "unit": "g"})),
    )
    .await;
    assert_eq!(status, StatusCode::CREATED);

    let recipe = create_recipe(&state, "Receta sal", "Sal", 0.2, "g").await;
    add_plan_entry(&state, "2026-03-02", &recipe, 2).await;

    let (status, groups) = common::request(
        state.clone(),
        "POST",
        "/api/shopping-list/from-plan",
        Some(json!({"start_date": "2026-03-02", "end_date": "2026-03-02"})),
    )
    .await;
    assert_eq!(status, StatusCode::OK, "respuesta: {groups}");

    let sal = find_item(&groups, "Sal").expect("sal fusionada");
    assert_eq!(sal["amount"], 0.3, "0.1 + 0.2 debe redondear a 0.3");
}

/// La fusión de unidades `Unknown` normaliza la clave: `Cajas` y `cajas` coinciden.
#[tokio::test]
async fn from_plan_merges_unknown_units_case_insensitively() {
    let state = common::test_state().await;

    let (status, _) = common::request(
        state.clone(),
        "POST",
        "/api/shopping-list",
        Some(json!({"name": "Huevos", "amount": 2.0, "unit": "cajas"})),
    )
    .await;
    assert_eq!(status, StatusCode::CREATED);

    let recipe = create_recipe(&state, "Receta huevos", "Huevos", 3.0, "Cajas").await;
    add_plan_entry(&state, "2026-03-02", &recipe, 2).await;

    let (status, groups) = common::request(
        state.clone(),
        "POST",
        "/api/shopping-list/from-plan",
        Some(json!({"start_date": "2026-03-02", "end_date": "2026-03-02"})),
    )
    .await;
    assert_eq!(status, StatusCode::OK, "respuesta: {groups}");

    let huevos = find_item(&groups, "Huevos").expect("huevos fusionados");
    assert_eq!(huevos["amount"], 5.0, "2 cajas + 3 cajas");
    let count: i64 =
        sqlx::query_scalar("SELECT count(*) FROM shopping_list_items WHERE name = 'Huevos'")
            .fetch_one(&state.pool)
            .await
            .expect("contar huevos");
    assert_eq!(count, 1, "no debe duplicar por mayúsculas");
}

/// Un aporte de 0.0 no reescribe la fila ni refresca `updated_at`.
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

    let (status, _) = common::request(
        state.clone(),
        "POST",
        "/api/shopping-list/from-plan",
        Some(json!({"start_date": "2026-03-02", "end_date": "2026-03-02"})),
    )
    .await;
    assert_eq!(status, StatusCode::OK);

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
}
