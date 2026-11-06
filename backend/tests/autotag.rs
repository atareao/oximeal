//! Tests de integración del etiquetado automático de recetas
//! (change `recipe-autotagging`, sección 2.1–2.7).
//!
//! Fase TDD **RED**: estos tests describen el comportamiento esperado del
//! etiquetado automático y, en el estado actual del código, DEBEN fallar (por
//! ausencia de la migración `0009`, del endpoint `POST /api/recipes/{id}/autotag`
//! o de la procedencia `source` en las etiquetas). No implementan nada.

mod common;

use axum::body::Body;
use axum::http::{Request, StatusCode, header};
use common::idp::{API_AUDIENCE, MockIdp, USER_GROUPS, access_token};
use common::{send, test_state, test_state_oidc};
use oximeal_backend::state::AppState;
use serde_json::{Value, json};
use sqlx::Row;

/// Crea una receta vía API y devuelve la `Recipe` devuelta (debe ser 201).
async fn create_recipe(state: &AppState, body: Value) -> Value {
    let (status, response) =
        common::request(state.clone(), "POST", "/api/recipes", Some(body)).await;
    assert_eq!(status, StatusCode::CREATED, "alta de receta: {response}");
    response
}

/// Cuerpo de receta mínimo con los ingredientes indicados.
fn payload(name: &str, category: &str, ingredients: Value) -> Value {
    json!({
        "name": name,
        "category": category,
        "ingredients": ingredients,
    })
}

/// Devuelve la etiqueta con el `slug` dado, si la receta la tiene.
fn tag<'a>(recipe: &'a Value, slug: &str) -> Option<&'a Value> {
    recipe["tags"]
        .as_array()?
        .iter()
        .find(|t| t["slug"] == slug)
}

/// `true` si la receta está asociada a la etiqueta `slug`.
fn has_tag(recipe: &Value, slug: &str) -> bool {
    tag(recipe, slug).is_some()
}

/// Procedencia (`source`) de una etiqueta de la receta, si está expuesta.
fn source_of(recipe: &Value, slug: &str) -> Option<String> {
    tag(recipe, slug)
        .and_then(|t| t.get("source"))
        .and_then(Value::as_str)
        .map(str::to_string)
}

/// Slugs de todas las etiquetas de la receta, ordenados.
fn slugs(recipe: &Value) -> Vec<String> {
    let mut result: Vec<String> = recipe["tags"]
        .as_array()
        .expect("array tags")
        .iter()
        .map(|t| t["slug"].as_str().expect("slug").to_string())
        .collect();
    result.sort();
    result
}

/// Número de supresiones registradas para una receta.
async fn suppression_count(state: &AppState, recipe_id: &str) -> i64 {
    sqlx::query_scalar("SELECT COUNT(*) FROM recipe_tag_suppressions WHERE recipe_id = ?")
        .bind(recipe_id)
        .fetch_one(&state.pool)
        .await
        .expect("contar supresiones")
}

/// Petición con Bearer opcional para los escenarios de permisos.
async fn call(
    state: &AppState,
    method: &str,
    uri: &str,
    bearer: Option<&str>,
    body: Option<Value>,
) -> StatusCode {
    let mut builder = Request::builder().method(method).uri(uri);
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

// ---------------------------------------------------------------------------
// 2.1 Migración `0009`: columna `source`, tabla de supresiones y catálogo.
// ---------------------------------------------------------------------------

#[tokio::test]
async fn migration_0009_adds_source_suppressions_and_auto_catalog() {
    let state = test_state().await;

    // La columna `recipe_tags.source` existe, es NOT NULL y por defecto `manual`.
    let columns = sqlx::query("PRAGMA table_info('recipe_tags')")
        .fetch_all(&state.pool)
        .await
        .expect("table_info recipe_tags");
    let source = columns
        .iter()
        .find(|row| row.get::<String, _>("name") == "source")
        .expect("falta la columna recipe_tags.source");
    assert_eq!(
        source.get::<i64, _>("notnull"),
        1,
        "recipe_tags.source debe ser NOT NULL"
    );
    let default: Option<String> = source.try_get("dflt_value").unwrap_or(None);
    assert!(
        default.as_deref().is_some_and(|d| d.contains("manual")),
        "recipe_tags.source debe tener DEFAULT 'manual', fue {default:?}"
    );

    // Existe la tabla de supresiones.
    let tables: Vec<String> =
        sqlx::query_scalar("SELECT name FROM sqlite_master WHERE type = 'table'")
            .fetch_all(&state.pool)
            .await
            .expect("sqlite_master: tablas");
    assert!(
        tables.iter().any(|t| t == "recipe_tag_suppressions"),
        "falta la tabla recipe_tag_suppressions: {tables:?}"
    );

    // El catálogo automático existe con sus cinco slugs canónicos.
    let auto_slugs: Vec<String> = sqlx::query_scalar(
        "SELECT slug FROM tags \
         WHERE slug IN ('vegetariano','vegano','sin-gluten','sin-lactosa','picante') \
         ORDER BY slug",
    )
    .fetch_all(&state.pool)
    .await
    .expect("catálogo automático");
    assert_eq!(
        auto_slugs,
        vec![
            "picante".to_string(),
            "sin-gluten".to_string(),
            "sin-lactosa".to_string(),
            "vegano".to_string(),
            "vegetariano".to_string(),
        ],
        "deben existir las cinco etiquetas automáticas"
    );
}

// ---------------------------------------------------------------------------
// 2.2 Heurística vía API.
// ---------------------------------------------------------------------------

#[tokio::test]
async fn vegan_recipe_by_absence_of_animal_origin_gets_vegano_and_vegetariano() {
    let state = test_state().await;

    let recipe = create_recipe(
        &state,
        payload(
            "Ensalada vegana",
            "salad",
            json!([
                {"name": "Tomate", "amount": 2.0, "unit": "ud"},
                {"name": "Lechuga", "amount": 1.0, "unit": "ud"}
            ]),
        ),
    )
    .await;

    assert!(has_tag(&recipe, "vegano"), "faltó vegano: {recipe}");
    assert!(
        has_tag(&recipe, "vegetariano"),
        "faltó vegetariano: {recipe}"
    );
}

#[tokio::test]
async fn recipe_with_meat_is_not_vegetarian_nor_vegan() {
    let state = test_state().await;

    let recipe = create_recipe(
        &state,
        payload(
            "Pollo con arroz",
            "main",
            json!([
                {"name": "Pollo", "amount": 200.0, "unit": "g"},
                {"name": "Arroz", "amount": 100.0, "unit": "g"}
            ]),
        ),
    )
    .await;

    assert!(
        !has_tag(&recipe, "vegetariano"),
        "no debe ser vegetariano: {recipe}"
    );
    assert!(!has_tag(&recipe, "vegano"), "no debe ser vegano: {recipe}");
    // La heurística sí emite otras exclusiones por ausencia: fuerza el RED.
    assert!(
        has_tag(&recipe, "sin-gluten"),
        "una receta con carne sin gluten debe recibir sin-gluten: {recipe}"
    );
}

#[tokio::test]
async fn recipe_without_gluten_gets_sin_gluten() {
    let state = test_state().await;

    let recipe = create_recipe(
        &state,
        payload(
            "Arroz blanco",
            "side",
            json!([{"name": "Arroz", "amount": 100.0, "unit": "g"}]),
        ),
    )
    .await;

    assert!(has_tag(&recipe, "sin-gluten"), "faltó sin-gluten: {recipe}");
}

#[tokio::test]
async fn recipe_with_spicy_ingredient_gets_picante() {
    let state = test_state().await;

    let recipe = create_recipe(
        &state,
        payload(
            "Salsa picante",
            "side",
            json!([{"name": "Chile", "amount": 1.0, "unit": "ud"}]),
        ),
    )
    .await;

    assert!(has_tag(&recipe, "picante"), "faltó picante: {recipe}");
}

#[tokio::test]
async fn panqueque_does_not_match_pan_and_keeps_sin_gluten() {
    let state = test_state().await;

    let recipe = create_recipe(
        &state,
        payload(
            "Panqueques",
            "dessert",
            json!([{"name": "Panqueque", "amount": 2.0, "unit": "ud"}]),
        ),
    )
    .await;

    // «panqueque» no debe contar como «pan»: sigue siendo sin gluten.
    assert!(
        has_tag(&recipe, "sin-gluten"),
        "panqueque no es pan, debe ser sin-gluten: {recipe}"
    );
}

// ---------------------------------------------------------------------------
// 2.3 El alta asigna automáticas; la manual gana; `auto_tags: false`.
// ---------------------------------------------------------------------------

#[tokio::test]
async fn manual_tag_wins_over_automatic_and_is_stored_once() {
    let state = test_state().await;

    let recipe = create_recipe(
        &state,
        json!({
            "name": "Ensalada",
            "category": "salad",
            "ingredients": [{"name": "Tomate", "amount": 2.0, "unit": "ud"}],
            "tags": ["Vegano"]
        }),
    )
    .await;

    let vegano = tag(&recipe, "vegano").expect("la receta debe tener vegano");
    assert_eq!(
        vegano["source"], "manual",
        "la etiqueta manual debe tener precedencia: {recipe}"
    );
    let occurrences = slugs(&recipe)
        .into_iter()
        .filter(|slug| slug == "vegano")
        .count();
    assert_eq!(occurrences, 1, "una sola fila para vegano: {recipe}");
    // La automática complementaria sí se añade.
    assert!(
        has_tag(&recipe, "vegetariano"),
        "faltó la automática vegetariano: {recipe}"
    );
}

#[tokio::test]
async fn auto_tags_false_disables_automatic_and_keeps_manual() {
    let state = test_state().await;

    let recipe = create_recipe(
        &state,
        json!({
            "name": "Ensalada",
            "category": "salad",
            "ingredients": [{"name": "Tomate", "amount": 2.0, "unit": "ud"}],
            "tags": ["Vegano"],
            "auto_tags": false
        }),
    )
    .await;

    assert_eq!(slugs(&recipe), vec!["vegano".to_string()]);
    let vegano = tag(&recipe, "vegano").expect("vegano manual");
    assert_eq!(
        vegano["source"], "manual",
        "sin automáticas, solo queda la manual: {recipe}"
    );
}

// ---------------------------------------------------------------------------
// 2.4 Supresión: impide reasignar; añadir manualmente la quita.
// ---------------------------------------------------------------------------

#[tokio::test]
async fn suppressed_tag_is_not_reassigned() {
    let state = test_state().await;

    let recipe = create_recipe(
        &state,
        json!({
            "name": "Ensalada",
            "category": "salad",
            "ingredients": [{"name": "Tomate", "amount": 2.0, "unit": "ud"}],
            "suppressed_auto_tags": ["sin-gluten"]
        }),
    )
    .await;

    assert!(
        has_tag(&recipe, "vegano"),
        "las no suprimidas siguen asignándose: {recipe}"
    );
    assert!(
        !has_tag(&recipe, "sin-gluten"),
        "la etiqueta suprimida no debe asignarse: {recipe}"
    );
}

#[tokio::test]
async fn adding_suppressed_tag_manually_removes_suppression() {
    let state = test_state().await;

    let recipe = create_recipe(
        &state,
        json!({
            "name": "Ensalada",
            "category": "salad",
            "ingredients": [{"name": "Tomate", "amount": 2.0, "unit": "ud"}],
            "tags": ["Sin gluten"],
            "suppressed_auto_tags": ["sin-gluten"]
        }),
    )
    .await;

    let id = recipe["id"].as_str().expect("id de receta").to_string();
    assert_eq!(
        source_of(&recipe, "sin-gluten").as_deref(),
        Some("manual"),
        "debe quedar asociada como manual: {recipe}"
    );
    assert_eq!(
        suppression_count(&state, &id).await,
        0,
        "la supresión debe desaparecer"
    );
}

// ---------------------------------------------------------------------------
// 2.5 `Recipe` y `RecipeSummary` exponen `source` en cada etiqueta.
// ---------------------------------------------------------------------------

#[tokio::test]
async fn recipe_and_summary_expose_tag_source() {
    let state = test_state().await;

    let recipe = create_recipe(
        &state,
        json!({
            "name": "Ensalada mixta",
            "category": "salad",
            "ingredients": [{"name": "Tomate", "amount": 2.0, "unit": "ud"}],
            "tags": ["Vegano"]
        }),
    )
    .await;
    let id = recipe["id"].as_str().expect("id de receta").to_string();

    // Detalle: cada etiqueta incluye `source` con su procedencia.
    let (status, detail) =
        common::request(state.clone(), "GET", &format!("/api/recipes/{id}"), None).await;
    assert_eq!(status, StatusCode::OK, "body: {detail}");
    for t in detail["tags"].as_array().expect("tags del detalle") {
        assert!(
            t.get("source").is_some(),
            "cada etiqueta debe exponer source: {t}"
        );
    }
    assert_eq!(
        source_of(&detail, "vegano").as_deref(),
        Some("manual"),
        "vegano manual en el detalle: {detail}"
    );
    assert_eq!(
        source_of(&detail, "vegetariano").as_deref(),
        Some("auto"),
        "vegetariano auto en el detalle: {detail}"
    );

    // Listado: el resumen también expone `source`.
    let (status, list) = common::request(state, "GET", "/api/recipes", None).await;
    assert_eq!(status, StatusCode::OK, "body: {list}");
    let item = &list["items"][0];
    for t in item["tags"].as_array().expect("tags del resumen") {
        assert!(
            t.get("source").is_some(),
            "cada etiqueta del resumen debe exponer source: {t}"
        );
    }
}

// ---------------------------------------------------------------------------
// 2.6 `POST /api/recipes/{id}/autotag`.
// ---------------------------------------------------------------------------

#[tokio::test]
async fn autotag_endpoint_recalculates_and_returns_recipe() {
    let state = test_state().await;

    let created = create_recipe(
        &state,
        payload(
            "Arroz blanco",
            "side",
            json!([{"name": "Arroz", "amount": 100.0, "unit": "g"}]),
        ),
    )
    .await;
    let id = created["id"].as_str().expect("id de receta").to_string();

    let (status, body) =
        common::request(state, "POST", &format!("/api/recipes/{id}/autotag"), None).await;
    assert_eq!(status, StatusCode::OK, "body: {body}");
    assert_eq!(body["id"], id);
    assert!(
        has_tag(&body, "sin-gluten"),
        "el recálculo devuelve la Recipe con sus etiquetas: {body}"
    );
}

#[tokio::test]
async fn autotag_endpoint_unknown_id_returns_404() {
    let state = test_state().await;

    let (status, _) = common::request(state, "POST", "/api/recipes/no-existe/autotag", None).await;
    assert_eq!(status, StatusCode::NOT_FOUND);
}

#[tokio::test]
async fn autotag_endpoint_without_credentials_returns_401() {
    let idp = MockIdp::start().await;
    let state = test_state_oidc(&idp.issuer, USER_GROUPS).await;

    let status = call(&state, "POST", "/api/recipes/whatever/autotag", None, None).await;
    assert_eq!(status, StatusCode::UNAUTHORIZED);
}

#[tokio::test]
async fn autotag_endpoint_without_permission_returns_403() {
    let idp = MockIdp::start().await;
    let state = test_state_oidc(&idp.issuer, USER_GROUPS).await;

    let token = access_token(&idp.issuer, "read:recipes", API_AUDIENCE);
    let status = call(
        &state,
        "POST",
        "/api/recipes/whatever/autotag",
        Some(&token),
        None,
    )
    .await;
    assert_eq!(status, StatusCode::FORBIDDEN);
}

// ---------------------------------------------------------------------------
// 2.7 Backfill único e idempotente.
// ---------------------------------------------------------------------------

/// Slugs de las etiquetas `auto` de una receta, ordenados.
async fn auto_slugs(state: &AppState, recipe_id: &str) -> Vec<String> {
    sqlx::query_scalar(
        "SELECT t.slug FROM recipe_tags rt JOIN tags t ON t.id = rt.tag_id \
         WHERE rt.recipe_id = ? AND rt.source = 'auto' ORDER BY t.slug",
    )
    .bind(recipe_id)
    .fetch_all(&state.pool)
    .await
    .expect("etiquetas auto")
}

/// El backfill (`services::autotag::backfill`) etiqueta las recetas sin filas
/// `auto` y es idempotente merced al marcador persistente (`PRAGMA user_version`).
#[tokio::test]
async fn backfill_is_single_and_idempotent() {
    let state = test_state().await;

    // Receta sin etiquetas automáticas (sembrada directamente en la BD para
    // simular un estado previo al backfill).
    sqlx::query(
        "INSERT INTO recipes (id, name, category, ingredients, servings) \
         VALUES ('backfill-1', 'Arroz blanco', 'side', \
                 '[{\"name\":\"Arroz\",\"amount\":100.0,\"unit\":\"g\"}]', 2)",
    )
    .execute(&state.pool)
    .await
    .expect("sembrar receta");

    // Primera pasada: calcula las automáticas por ausencia (arroz: sin gluten,
    // sin lactosa, vegetariano y vegano).
    oximeal_backend::services::autotag::backfill(&state.pool)
        .await
        .expect("primer backfill");
    let first = auto_slugs(&state, "backfill-1").await;
    assert!(
        first.contains(&"sin-gluten".to_string()),
        "faltó sin-gluten: {first:?}"
    );
    assert!(
        first.contains(&"vegano".to_string()),
        "faltó vegano: {first:?}"
    );

    // Segunda pasada: el marcador persistente evita repetir; el conjunto no cambia.
    oximeal_backend::services::autotag::backfill(&state.pool)
        .await
        .expect("segundo backfill");
    assert_eq!(first, auto_slugs(&state, "backfill-1").await);
}

/// Una receta con `ingredients` corrupto se omite con log y el backfill sigue
/// con el resto, sin abortar el arranque.
#[tokio::test]
async fn backfill_skips_corrupt_ingredients_and_continues() {
    let state = test_state().await;

    sqlx::query(
        "INSERT INTO recipes (id, name, category, ingredients, servings) \
         VALUES ('backfill-bad', 'Receta corrupta', 'side', 'no-es-json', 2)",
    )
    .execute(&state.pool)
    .await
    .expect("sembrar receta corrupta");

    sqlx::query(
        "INSERT INTO recipes (id, name, category, ingredients, servings) \
         VALUES ('backfill-good', 'Arroz blanco', 'side', \
                 '[{\"name\":\"Arroz\",\"amount\":100.0,\"unit\":\"g\"}]', 2)",
    )
    .execute(&state.pool)
    .await
    .expect("sembrar receta válida");

    oximeal_backend::services::autotag::backfill(&state.pool)
        .await
        .expect("el backfill debe tolerar la fila corrupta");

    // La fila mala se salta (sin etiquetas `auto`) y la buena se procesa.
    assert!(
        auto_slugs(&state, "backfill-bad").await.is_empty(),
        "la receta corrupta no debe recibir etiquetas"
    );
    let good = auto_slugs(&state, "backfill-good").await;
    assert!(
        good.contains(&"sin-gluten".to_string()),
        "el backfill debe continuar con la receta válida: {good:?}"
    );
}

/// Si una fila no se puede procesar, el marcador persistente NO debe avanzar:
/// así el siguiente arranque reintenta las recetas pendientes en lugar de
/// darlas por etiquetadas para siempre.
#[tokio::test]
async fn backfill_does_not_advance_marker_when_a_row_fails() {
    let state = test_state().await;

    sqlx::query(
        "INSERT INTO recipes (id, name, category, ingredients, servings) \
         VALUES ('backfill-bad', 'Receta corrupta', 'side', 'no-es-json', 2)",
    )
    .execute(&state.pool)
    .await
    .expect("sembrar receta corrupta");

    sqlx::query(
        "INSERT INTO recipes (id, name, category, ingredients, servings) \
         VALUES ('backfill-good', 'Arroz blanco', 'side', \
                 '[{\"name\":\"Arroz\",\"amount\":100.0,\"unit\":\"g\"}]', 2)",
    )
    .execute(&state.pool)
    .await
    .expect("sembrar receta válida");

    let version_before: i64 = sqlx::query_scalar("PRAGMA user_version")
        .fetch_one(&state.pool)
        .await
        .expect("leer user_version inicial");
    assert!(
        version_before < oximeal_backend::services::autotag::BACKFILL_VERSION,
        "el test requiere partir de un marcador sin avanzar"
    );

    oximeal_backend::services::autotag::backfill(&state.pool)
        .await
        .expect("el backfill debe tolerar la fila corrupta");

    // La fila buena sí se procesa, pero el marcador no se avanza porque hubo
    // una receta fallida.
    assert!(
        auto_slugs(&state, "backfill-good")
            .await
            .contains(&"sin-gluten".to_string()),
        "la receta válida debe procesarse pese al fallo de otra fila"
    );
    let version_after: i64 = sqlx::query_scalar("PRAGMA user_version")
        .fetch_one(&state.pool)
        .await
        .expect("leer user_version tras backfill con fallos");
    assert_eq!(
        version_after, version_before,
        "el marcador no debe avanzar si alguna receta falló"
    );

    // Un segundo arranque vuelve a intentar el backfill (la receta buena ya no
    // está pendiente, pero la corrupta sigue bloqueando el marcador).
    oximeal_backend::services::autotag::backfill(&state.pool)
        .await
        .expect("el segundo backfill también debe tolerar la fila corrupta");
    let version_after_second: i64 = sqlx::query_scalar("PRAGMA user_version")
        .fetch_one(&state.pool)
        .await
        .expect("leer user_version tras el segundo backfill");
    assert_eq!(
        version_after_second, version_before,
        "el marcador sigue sin avanzar mientras persista la receta fallida"
    );
}
