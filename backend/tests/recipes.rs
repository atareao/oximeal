//! Tests de integración de los endpoints de recetas.

mod common;

use axum::http::StatusCode;
use serde_json::{Value, json};

/// Cuerpo de receta válido y parametrizable.
fn recipe_payload(name: &str, category: &str) -> Value {
    json!({
        "name": name,
        "category": category,
        "diet_type": "mediterranean",
        "prep_time_minutes": 30,
        "servings": 2,
        "ingredients": [
            {"name": "Tomate", "amount": 2.0, "unit": "ud"}
        ],
        "instructions": "Cortar y servir.",
        "source_url": null
    })
}

#[tokio::test]
async fn create_valid_recipe_returns_201_and_detail() {
    let state = common::test_state().await;

    let (status, body) = common::request(
        state.clone(),
        "POST",
        "/api/recipes",
        Some(recipe_payload("Gazpacho andaluz", "salad")),
    )
    .await;

    assert_eq!(status, StatusCode::CREATED);
    assert!(body["id"].is_string(), "debe devolver id");
    assert_eq!(body["name"], "Gazpacho andaluz");
    assert_eq!(body["category"], "salad");
    assert_eq!(body["servings"], 2);
    assert_eq!(body["prep_time_minutes"], 30);
    assert_eq!(body["ingredients"].as_array().map(Vec::len), Some(1));

    // El detalle por id devuelve la receta completa.
    let id = body["id"].as_str().expect("id de receta");
    let (status, detail) = common::request(state, "GET", &format!("/api/recipes/{id}"), None).await;
    assert_eq!(status, StatusCode::OK);
    assert_eq!(detail["id"], id);
    assert_eq!(detail["instructions"], "Cortar y servir.");
}

#[tokio::test]
async fn invalid_category_returns_400_and_persists_nothing() {
    let state = common::test_state().await;

    let (status, _) = common::request(
        state.clone(),
        "POST",
        "/api/recipes",
        Some(recipe_payload("Receta rara", "invalid")),
    )
    .await;
    assert_eq!(status, StatusCode::BAD_REQUEST);

    let (status, list) = common::request(state, "GET", "/api/recipes", None).await;
    assert_eq!(status, StatusCode::OK);
    assert_eq!(list["total"], 0, "no debe persistirse ninguna receta");
}

#[tokio::test]
async fn create_recipe_without_ingredients_returns_400() {
    let state = common::test_state().await;

    let (status, _) = common::request(
        state,
        "POST",
        "/api/recipes",
        Some(json!({
            "name": "Vacía",
            "category": "main",
            "ingredients": []
        })),
    )
    .await;
    assert_eq!(status, StatusCode::BAD_REQUEST);
}

#[tokio::test]
async fn get_unknown_recipe_returns_404() {
    let state = common::test_state().await;

    let (status, _) = common::request(state, "GET", "/api/recipes/no-existe", None).await;
    assert_eq!(status, StatusCode::NOT_FOUND);
}

#[tokio::test]
async fn search_with_three_or_more_chars_uses_fts_over_instructions() {
    let state = common::test_state().await;

    let (status, _) = common::request(
        state.clone(),
        "POST",
        "/api/recipes",
        Some(json!({
            "name": "Lentejas estofadas",
            "category": "main",
            "ingredients": [{"name": "Lenteja", "amount": 200.0, "unit": "g"}],
            "instructions": "Con zanahoria y puerro"
        })),
    )
    .await;
    assert_eq!(status, StatusCode::CREATED);

    // "zana" solo aparece en las instrucciones y no es prefijo/sufijo del nombre.
    let (status, body) = common::request(state, "GET", "/api/recipes?q=zana", None).await;
    assert_eq!(status, StatusCode::OK);
    assert_eq!(body["total"], 1);
    assert_eq!(body["items"][0]["name"], "Lentejas estofadas");
    assert_eq!(body["page"], 1);
    assert_eq!(body["page_size"], 20);
}

#[tokio::test]
async fn short_search_uses_like_fallback() {
    let state = common::test_state().await;

    let (status, _) = common::request(
        state.clone(),
        "POST",
        "/api/recipes",
        Some(recipe_payload("Pollo asado", "main")),
    )
    .await;
    assert_eq!(status, StatusCode::CREATED);

    // Query de 2 caracteres: no puede usar trigram, debe caer a LIKE.
    let (status, body) = common::request(state, "GET", "/api/recipes?q=ll", None).await;
    assert_eq!(status, StatusCode::OK);
    assert_eq!(body["total"], 1);
    assert_eq!(body["items"][0]["name"], "Pollo asado");
}

#[tokio::test]
async fn category_filter_returns_only_matching() {
    let state = common::test_state().await;

    common::request(
        state.clone(),
        "POST",
        "/api/recipes",
        Some(recipe_payload("Tarta de queso", "dessert")),
    )
    .await;
    common::request(
        state.clone(),
        "POST",
        "/api/recipes",
        Some(recipe_payload("Pollo asado", "main")),
    )
    .await;

    let (status, body) = common::request(state, "GET", "/api/recipes?category=dessert", None).await;
    assert_eq!(status, StatusCode::OK);
    assert_eq!(body["total"], 1);
    assert_eq!(body["items"][0]["name"], "Tarta de queso");
}

#[tokio::test]
async fn invalid_category_filter_returns_400() {
    let state = common::test_state().await;

    let (status, _) = common::request(state, "GET", "/api/recipes?category=fruta", None).await;
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

/// F1 — El término de búsqueda va como query language de FTS5 y un carácter
/// especial produce un error de sintaxis → 500. Debe responder 200 siempre.
#[tokio::test]
async fn fts_special_characters_never_return_500() {
    let state = common::test_state().await;

    // Sembrar una receta para que la búsqueda FTS tenga contenido.
    let (status, _) = common::request(
        state.clone(),
        "POST",
        "/api/recipes",
        Some(recipe_payload("Tomate frito", "main")),
    )
    .await;
    assert_eq!(status, StatusCode::CREATED);

    // Términos de >= 3 caracteres que FTS5 interpretaría como query language.
    for q in [
        "tomate AND",
        "a:b",
        "(((",
        "\"\"\"",
        "foo:bar",
        "NOT *",
        "OR",
    ] {
        let uri = format!("/api/recipes?q={}", encode_query(q));
        let (status, _) = common::request(state.clone(), "GET", &uri, None).await;
        assert_ne!(
            status,
            StatusCode::INTERNAL_SERVER_ERROR,
            "q={q:?} no debe producir 500"
        );
        assert_eq!(status, StatusCode::OK, "q={q:?} debe responder 200");
    }

    // Un término normal sigue encontrando la receta.
    let (status, body) = common::request(state, "GET", "/api/recipes?q=tomate", None).await;
    assert_eq!(status, StatusCode::OK);
    assert_eq!(body["total"], 1, "q=tomate debe seguir encontrando");
}

/// F4 — Un ingrediente con nombre vacío invalida la receta (400) sin persistir.
#[tokio::test]
async fn create_recipe_with_empty_ingredient_name_returns_400() {
    let state = common::test_state().await;

    let (status, _) = common::request(
        state.clone(),
        "POST",
        "/api/recipes",
        Some(json!({
            "name": "Receta con ingrediente vacío",
            "category": "main",
            "ingredients": [{"name": "   ", "amount": 1.0, "unit": "ud"}]
        })),
    )
    .await;
    assert_eq!(status, StatusCode::BAD_REQUEST);

    let (status, list) = common::request(state, "GET", "/api/recipes", None).await;
    assert_eq!(status, StatusCode::OK);
    assert_eq!(list["total"], 0, "no debe persistirse ninguna receta");
}

/// F4 — Un ingrediente con `amount` negativo invalida la receta (400).
#[tokio::test]
async fn create_recipe_with_negative_ingredient_amount_returns_400() {
    let state = common::test_state().await;

    let (status, _) = common::request(
        state.clone(),
        "POST",
        "/api/recipes",
        Some(json!({
            "name": "Receta con amount negativo",
            "category": "main",
            "ingredients": [{"name": "Tomate", "amount": -1.0, "unit": "ud"}]
        })),
    )
    .await;
    assert_eq!(status, StatusCode::BAD_REQUEST);

    let (status, list) = common::request(state, "GET", "/api/recipes", None).await;
    assert_eq!(status, StatusCode::OK);
    assert_eq!(list["total"], 0, "no debe persistirse ninguna receta");
}

/// Caracterización — el término es una frase FTS: una subcadena contigua
/// multi-palabra (aunque cruce fronteras de palabras) debe coincidir.
#[tokio::test]
async fn search_matches_contiguous_substring_across_words() {
    let state = common::test_state().await;

    let (status, _) = common::request(
        state.clone(),
        "POST",
        "/api/recipes",
        Some(json!({
            "name": "Guiso",
            "category": "main",
            "ingredients": [{"name": "Zanahoria", "amount": 1.0, "unit": "ud"}],
            "instructions": "lentejas estofadas con zanahoria"
        })),
    )
    .await;
    assert_eq!(status, StatusCode::CREATED);

    // "estofadas con" es una subcadena contigua de las instrucciones.
    let uri = format!("/api/recipes?q={}", encode_query("estofadas con"));
    let (status, body) = common::request(state, "GET", &uri, None).await;
    assert_eq!(status, StatusCode::OK);
    assert_eq!(
        body["total"], 1,
        "subcadena contigua multi-palabra debe coincidir"
    );
    assert_eq!(body["items"][0]["name"], "Guiso");
}

/// Caracterización — al ser una frase contigua, palabras no adyacentes con
/// texto intercalado NO coinciden.
#[tokio::test]
async fn search_does_not_match_non_contiguous_words() {
    let state = common::test_state().await;

    let (status, _) = common::request(
        state.clone(),
        "POST",
        "/api/recipes",
        Some(json!({
            "name": "Guiso",
            "category": "main",
            "ingredients": [{"name": "Zanahoria", "amount": 1.0, "unit": "ud"}],
            "instructions": "lentejas estofadas con zanahoria"
        })),
    )
    .await;
    assert_eq!(status, StatusCode::CREATED);

    // "lentejas zanahoria" NO es una subcadena contigua: la frase no coincide.
    let uri = format!("/api/recipes?q={}", encode_query("lentejas zanahoria"));
    let (status, body) = common::request(state, "GET", &uri, None).await;
    assert_eq!(status, StatusCode::OK);
    assert_eq!(body["total"], 0, "palabras no contiguas no deben coincidir");
}

#[tokio::test]
async fn search_three_char_term_in_instructions_uses_fts() {
    let state = common::test_state().await;

    let (status, _) = common::request(
        state.clone(),
        "POST",
        "/api/recipes",
        Some(json!({
            "name": "Guiso",
            "category": "main",
            "ingredients": [{"name": "Puerro", "amount": 1.0, "unit": "ud"}],
            "instructions": "salteado con puerro"
        })),
    )
    .await;
    assert_eq!(status, StatusCode::CREATED);

    // "uer" (3 chars) solo aparece en las instrucciones: si la búsqueda no usara
    // FTS (>= 3 caracteres), el `LIKE` sobre el nombre no lo encontraría.
    let uri = format!("/api/recipes?q={}", encode_query("uer"));
    let (status, body) = common::request(state, "GET", &uri, None).await;
    assert_eq!(status, StatusCode::OK);
    assert_eq!(
        body["total"], 1,
        "un término de 3 chars debe buscar también en instrucciones (FTS)"
    );
}

/// `PUT /api/recipes/{id}` válido → 200 con datos nuevos y FTS sincronizado.
#[tokio::test]
async fn update_valid_recipe_returns_200_and_fts_reflects_change() {
    let state = common::test_state().await;

    let (status, created) = common::request(
        state.clone(),
        "POST",
        "/api/recipes",
        Some(recipe_payload("Gazpacho andaluz", "salad")),
    )
    .await;
    assert_eq!(status, StatusCode::CREATED);
    let id = created["id"].as_str().expect("id de receta").to_string();

    let (status, updated) = common::request(
        state.clone(),
        "PUT",
        &format!("/api/recipes/{id}"),
        Some(json!({
            "name": "Salmorejo cordobés",
            "category": "main",
            "diet_type": "mediterranean",
            "prep_time_minutes": 15,
            "servings": 4,
            "ingredients": [{"name": "Pan", "amount": 200.0, "unit": "g"}],
            "instructions": "Triturar todo.",
            "source_url": null
        })),
    )
    .await;
    assert_eq!(status, StatusCode::OK, "body: {updated}");
    assert_eq!(updated["id"], id);
    assert_eq!(updated["name"], "Salmorejo cordobés");
    assert_eq!(updated["category"], "main");
    assert_eq!(updated["servings"], 4);
    assert_eq!(updated["prep_time_minutes"], 15);
    assert_eq!(updated["ingredients"][0]["name"], "Pan");
    assert_eq!(updated["instructions"], "Triturar todo.");

    // FTS sincronizado: el nombre NUEVO encuentra la receta...
    let uri = format!("/api/recipes?q={}", encode_query("Salmorejo"));
    let (status, body) = common::request(state.clone(), "GET", &uri, None).await;
    assert_eq!(status, StatusCode::OK);
    assert_eq!(body["total"], 1, "el nombre nuevo debe ser encontrable");
    assert_eq!(body["items"][0]["name"], "Salmorejo cordobés");

    // ...y el nombre ANTIGUO ya no aparece.
    let uri = format!("/api/recipes?q={}", encode_query("Gazpacho"));
    let (status, body) = common::request(state, "GET", &uri, None).await;
    assert_eq!(status, StatusCode::OK);
    assert_eq!(body["total"], 0, "el nombre antiguo no debe encontrarse");
}

/// `PUT /api/recipes/{id}` con id inexistente → 404 y no crea nada.
#[tokio::test]
async fn update_unknown_recipe_returns_404_and_creates_nothing() {
    let state = common::test_state().await;

    let (status, _) = common::request(
        state.clone(),
        "PUT",
        "/api/recipes/no-existe",
        Some(recipe_payload("Nueva receta", "main")),
    )
    .await;
    assert_eq!(status, StatusCode::NOT_FOUND);

    let (status, list) = common::request(state, "GET", "/api/recipes", None).await;
    assert_eq!(status, StatusCode::OK);
    assert_eq!(list["total"], 0, "no debe crearse ninguna receta");
}

/// `PUT /api/recipes/{id}` con payload inválido → 400 y la receta original intacta.
#[tokio::test]
async fn update_with_invalid_payload_returns_400_and_keeps_original() {
    let state = common::test_state().await;

    let (status, created) = common::request(
        state.clone(),
        "POST",
        "/api/recipes",
        Some(recipe_payload("Original", "main")),
    )
    .await;
    assert_eq!(status, StatusCode::CREATED);
    let id = created["id"].as_str().expect("id de receta").to_string();

    let invalid_payloads = [
        // Categoría no permitida.
        recipe_payload("Cambiada", "fruta"),
        // Nombre vacío.
        json!({
            "name": "   ",
            "category": "main",
            "ingredients": [{"name": "Tomate", "amount": 1.0, "unit": "ud"}]
        }),
        // Sin ingredientes.
        json!({
            "name": "Sin ingredientes",
            "category": "main",
            "ingredients": []
        }),
    ];

    for payload in invalid_payloads {
        let (status, _) = common::request(
            state.clone(),
            "PUT",
            &format!("/api/recipes/{id}"),
            Some(payload),
        )
        .await;
        assert_eq!(status, StatusCode::BAD_REQUEST);
    }

    let (status, detail) = common::request(state, "GET", &format!("/api/recipes/{id}"), None).await;
    assert_eq!(status, StatusCode::OK);
    assert_eq!(detail["name"], "Original", "la receta no debe cambiar");
    assert_eq!(detail["category"], "main");
}

/// `DELETE /api/recipes/{id}` válido → 204, el detalle pasa a 404 y sale del listado.
#[tokio::test]
async fn delete_valid_recipe_returns_204_and_removes_from_list() {
    let state = common::test_state().await;

    let (status, created) = common::request(
        state.clone(),
        "POST",
        "/api/recipes",
        Some(recipe_payload("Para borrar", "main")),
    )
    .await;
    assert_eq!(status, StatusCode::CREATED);
    let id = created["id"].as_str().expect("id de receta").to_string();

    let (status, _) =
        common::request(state.clone(), "DELETE", &format!("/api/recipes/{id}"), None).await;
    assert_eq!(status, StatusCode::NO_CONTENT);

    let (status, _) =
        common::request(state.clone(), "GET", &format!("/api/recipes/{id}"), None).await;
    assert_eq!(status, StatusCode::NOT_FOUND, "el detalle debe dar 404");

    let (status, list) = common::request(state, "GET", "/api/recipes", None).await;
    assert_eq!(status, StatusCode::OK);
    assert_eq!(list["total"], 0, "no debe aparecer en el listado");
}

/// `DELETE /api/recipes/{id}` con id inexistente → 404.
#[tokio::test]
async fn delete_unknown_recipe_returns_404() {
    let state = common::test_state().await;

    let (status, _) = common::request(state, "DELETE", "/api/recipes/no-existe", None).await;
    assert_eq!(status, StatusCode::NOT_FOUND);
}

/// `DELETE` de una receta referenciada por el plan → 409 y nada cambia.
#[tokio::test]
async fn delete_recipe_referenced_by_plan_returns_409_and_changes_nothing() {
    let state = common::test_state().await;

    let (status, created) = common::request(
        state.clone(),
        "POST",
        "/api/recipes",
        Some(recipe_payload("En el plan", "main")),
    )
    .await;
    assert_eq!(status, StatusCode::CREATED);
    let id = created["id"].as_str().expect("id de receta").to_string();

    let (status, entry) = common::request(
        state.clone(),
        "POST",
        "/api/meal-plan",
        Some(json!({
            "date": "2024-01-15",
            "meal_type": "lunch",
            "recipe_id": id,
            "role": "main"
        })),
    )
    .await;
    assert_eq!(status, StatusCode::CREATED, "body: {entry}");

    let (status, body) =
        common::request(state.clone(), "DELETE", &format!("/api/recipes/{id}"), None).await;
    assert_eq!(status, StatusCode::CONFLICT, "body: {body}");

    // La receta sigue existiendo.
    let (status, _) =
        common::request(state.clone(), "GET", &format!("/api/recipes/{id}"), None).await;
    assert_eq!(status, StatusCode::OK, "la receta debe permanecer");

    // Y la entrada del plan también.
    let (status, plan) = common::request(
        state,
        "GET",
        "/api/meal-plan?start_date=2024-01-15&end_date=2024-01-15",
        None,
    )
    .await;
    assert_eq!(status, StatusCode::OK);
    assert_eq!(
        plan.as_array().map(Vec::len),
        Some(1),
        "la entrada del plan debe permanecer"
    );
}

#[tokio::test]
async fn short_query_searches_only_name_not_instructions() {
    let state = common::test_state().await;

    let (status, _) = common::request(
        state.clone(),
        "POST",
        "/api/recipes",
        Some(json!({
            "name": "Guiso",
            "category": "main",
            "ingredients": [{"name": "Puerro", "amount": 1.0, "unit": "ud"}],
            "instructions": "salteado con puerro"
        })),
    )
    .await;
    assert_eq!(status, StatusCode::CREATED);

    // "ue" (2 chars) está en las instrucciones pero el fallback LIKE solo mira
    // el nombre de la receta: no debe encontrarla.
    let uri = format!("/api/recipes?q={}", encode_query("ue"));
    let (status, body) = common::request(state, "GET", &uri, None).await;
    assert_eq!(status, StatusCode::OK);
    assert_eq!(
        body["total"], 0,
        "con < 3 caracteres solo se busca en el nombre"
    );
}
