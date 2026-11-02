//! Repositorio de recetas (SQLite vía `sqlx`, queries en runtime).

use sqlx::{QueryBuilder, Sqlite, SqlitePool};

use crate::error::{AppError, AppResult};
use crate::models::recipe::{
    Recipe, RecipeDetailRow, RecipeInput, RecipeListResponse, RecipeSummaryRow,
};

/// Longitud mínima de `q` para usar FTS5 (el tokenizer `trigram` exige 3).
const FTS_MIN_CHARS: usize = 3;

/// Lista recetas aplicando búsqueda, filtro de categoría y paginación.
///
/// # Semántica de la búsqueda
///
/// El término `q` se envuelve entre comillas (frase) y se envía a
/// `recipes_fts MATCH` sobre el nombre **o** las instrucciones cuando tiene 3 o
/// más caracteres. Con el tokenizer `trigram`, una frase coincide con una
/// **subcadena contigua** del texto indexado, insensible a mayúsculas y a
/// diacríticos; **no** es una búsqueda por palabras (AND): `"lentejas
/// zanahoria"` no encuentra "lentejas estofadas con zanahoria", pero
/// `"estofadas con"` sí.
///
/// Con menos de 3 caracteres (mínimo que exige el tokenizer `trigram`) se cae a
/// `LIKE '%q%'` sobre el nombre. Ver `docs/search.md`.
///
/// La validación de `category` corresponde a la capa de rutas.
pub async fn list(
    pool: &SqlitePool,
    q: Option<&str>,
    category: Option<&str>,
    page: u32,
    page_size: u32,
) -> AppResult<RecipeListResponse> {
    let search = q.map(str::trim).filter(|value| !value.is_empty());
    let use_fts = search.is_some_and(|value| value.chars().count() >= FTS_MIN_CHARS);

    let columns = "SELECT r.id, r.name, r.category, r.diet_type, \
                   r.prep_time_minutes, COALESCE(r.servings, 2) AS servings, \
                   CAST(r.created_at AS TEXT) AS created_at FROM recipes r";

    let mut data = QueryBuilder::<Sqlite>::new(columns);
    let mut count = QueryBuilder::<Sqlite>::new("SELECT COUNT(*) FROM recipes r");

    if use_fts {
        let join = " JOIN recipes_fts f ON f.rowid = r.rowid";
        data.push(join);
        count.push(join);
    }

    data.push(" WHERE 1 = 1");
    count.push(" WHERE 1 = 1");

    if let Some(term) = search {
        if use_fts {
            // FTS5 interpreta el término como *query language*; se envuelve en
            // una frase y se escapan las comillas dobles internas para que
            // caracteres como `"`, `(`, `:` o `AND` no provoquen error de sintaxis.
            let safe = format!("\"{}\"", term.replace('"', "\"\""));
            data.push(" AND recipes_fts MATCH ").push_bind(safe.clone());
            count.push(" AND recipes_fts MATCH ").push_bind(safe);
        } else {
            let pattern = format!("%{term}%");
            data.push(" AND r.name LIKE ").push_bind(pattern.clone());
            count.push(" AND r.name LIKE ").push_bind(pattern);
        }
    }

    if let Some(category) = category {
        data.push(" AND r.category = ")
            .push_bind(category.to_string());
        count
            .push(" AND r.category = ")
            .push_bind(category.to_string());
    }

    let total: i64 = count.build_query_scalar().fetch_one(pool).await?;

    let offset = i64::from(page.saturating_sub(1)) * i64::from(page_size);
    data.push(" ORDER BY r.created_at DESC, r.id ASC")
        .push(" LIMIT ")
        .push_bind(i64::from(page_size))
        .push(" OFFSET ")
        .push_bind(offset);

    let rows: Vec<RecipeSummaryRow> = data.build_query_as().fetch_all(pool).await?;
    let items = rows
        .into_iter()
        .map(RecipeSummaryRow::into_summary)
        .collect();

    Ok(RecipeListResponse {
        items,
        total,
        page,
        page_size,
    })
}

/// Inserta una receta y devuelve su representación completa.
pub async fn create(pool: &SqlitePool, id: &str, input: &RecipeInput) -> AppResult<Recipe> {
    let diet_type = input
        .diet_type
        .clone()
        .unwrap_or_else(|| "mediterranean".to_string());
    let servings = input.servings.unwrap_or(2);
    let ingredients = serde_json::to_string(&input.ingredients).map_err(|error| {
        AppError::Internal(anyhow::anyhow!("serializando ingredientes: {error}"))
    })?;

    sqlx::query(
        "INSERT INTO recipes \
         (id, name, category, diet_type, prep_time_minutes, servings, ingredients, instructions, source_url) \
         VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?)",
    )
    .bind(id)
    .bind(&input.name)
    .bind(&input.category)
    .bind(&diet_type)
    .bind(input.prep_time_minutes)
    .bind(servings)
    .bind(ingredients)
    .bind(&input.instructions)
    .bind(&input.source_url)
    .execute(pool)
    .await?;

    get(pool, id).await
}

/// Obtiene una receta completa por id; devuelve 404 si no existe.
pub async fn get(pool: &SqlitePool, id: &str) -> AppResult<Recipe> {
    let row: Option<RecipeDetailRow> = sqlx::query_as(
        "SELECT id, name, category, diet_type, prep_time_minutes, \
         COALESCE(servings, 2) AS servings, CAST(created_at AS TEXT) AS created_at, \
         CAST(ingredients AS TEXT) AS ingredients, instructions, source_url \
         FROM recipes WHERE id = ?",
    )
    .bind(id)
    .fetch_optional(pool)
    .await?;

    match row {
        Some(row) => row.into_recipe(),
        None => Err(AppError::NotFound(format!("receta {id}"))),
    }
}
