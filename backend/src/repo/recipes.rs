//! Repositorio de recetas (SQLite vía `sqlx`, queries en runtime).

use anyhow::anyhow;
use sqlx::{QueryBuilder, Sqlite, SqlitePool};

use crate::domain::ingredient::IngredientInput;
use crate::error::{AppError, AppResult};
use crate::models::recipe::{
    Recipe, RecipeDetailRow, RecipeInput, RecipeListResponse, RecipeSummaryRow,
};
use crate::repo::tags;
use crate::services::autotag;

/// Longitud mínima de `q` para usar FTS5 (el tokenizer `trigram` exige 3).
const FTS_MIN_CHARS: usize = 3;

/// Lista recetas aplicando búsqueda, filtros de categoría y etiqueta y paginación.
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
/// # Filtro por etiqueta
///
/// `tag` acepta el `id` **o** el `slug` de una etiqueta y restringe el resultado
/// a las recetas asociadas a ella. Las etiquetas **no** participan en la
/// búsqueda de texto: `q` solo mira `name` e `instructions`. El `total` refleja
/// el conjunto que cumple todos los filtros.
///
/// La validación de `category` corresponde a la capa de rutas.
pub async fn list(
    pool: &SqlitePool,
    q: Option<&str>,
    category: Option<&str>,
    tag: Option<&str>,
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

    if let Some(tag) = tag {
        // Resolución por id o slug; un tag inexistente no aporta filas (200 vacío).
        // Los marcadores se añaden con `push_bind` (no como literales `?`) porque
        // el constructor de consultas no admite `?` embebidos en texto.
        let head = " AND r.id IN (SELECT rt.recipe_id FROM recipe_tags rt \
                    JOIN tags t ON t.id = rt.tag_id WHERE t.id = ";
        let tail = " OR t.slug = ";
        data.push(head)
            .push_bind(tag.to_string())
            .push(tail)
            .push_bind(tag.to_string())
            .push(")");
        count
            .push(head)
            .push_bind(tag.to_string())
            .push(tail)
            .push_bind(tag.to_string())
            .push(")");
    }

    let total: i64 = count.build_query_scalar().fetch_one(pool).await?;

    let offset = i64::from(page.saturating_sub(1)) * i64::from(page_size);
    data.push(" ORDER BY r.created_at DESC, r.id ASC")
        .push(" LIMIT ")
        .push_bind(i64::from(page_size))
        .push(" OFFSET ")
        .push_bind(offset);

    let rows: Vec<RecipeSummaryRow> = data.build_query_as().fetch_all(pool).await?;

    // Una sola consulta para las etiquetas de toda la página (sin N+1).
    let ids: Vec<String> = rows.iter().map(|row| row.id.clone()).collect();
    let mut tags_by_recipe = tags::for_recipes(pool, &ids).await?;

    let items = rows
        .into_iter()
        .map(|row| {
            let tags = tags_by_recipe.remove(&row.id).unwrap_or_default();
            row.into_summary(tags)
        })
        .collect();

    Ok(RecipeListResponse {
        items,
        total,
        page,
        page_size,
    })
}

/// Inserta una receta y sus etiquetas y devuelve su representación completa.
///
/// La receta y su conjunto de etiquetas se persisten en la misma transacción
/// `BEGIN IMMEDIATE`.
pub async fn create(pool: &SqlitePool, id: &str, input: &RecipeInput) -> AppResult<Recipe> {
    let mut tx = pool.begin_with("BEGIN IMMEDIATE").await?;
    insert(&mut *tx, id, input).await?;
    tags::replace_for_recipe(&mut tx, id, &input.tags).await?;
    autotag::apply(&mut tx, id, input).await?;
    tx.commit().await?;
    get(pool, id).await
}

/// Crea una receta de forma idempotente por `source_url`.
///
/// Dentro de una transacción `BEGIN IMMEDIATE` busca una receta con la misma
/// `source_url`; si existe, la devuelve con `false` sin crear nada. Si no (o si
/// `source_url` es `None`), inserta `input` y devuelve la nueva con `true`.
/// Reutiliza la misma sentencia de inserción que [`create`].
///
/// # Limitación conocida
///
/// No existe un índice `UNIQUE` sobre `source_url`: la idempotencia es atómica
/// por el `BEGIN IMMEDIATE`, que serializa a los escritores de SQLite y cierra
/// la ventana de carrera entre el `SELECT` y el `INSERT`. Añadir un índice único
/// sobre datos de usuario ya existentes podría romper upgrades (duplicados
/// previos) sin aportar garantía adicional frente a writers concurrentes.
/// Si en el futuro se normaliza `source_url` (p. ej. quitando el fragmento), el
/// índice único sí sería la mejor defensa.
pub async fn create_or_get_by_source_url(
    pool: &SqlitePool,
    id: &str,
    input: &RecipeInput,
) -> AppResult<(Recipe, bool)> {
    // Escritura read-then-write: BEGIN IMMEDIATE evita SQLITE_BUSY_SNAPSHOT en WAL.
    let mut tx = pool.begin_with("BEGIN IMMEDIATE").await?;

    if let Some(source_url) = input.source_url.as_deref() {
        let existing: Option<String> =
            sqlx::query_scalar("SELECT id FROM recipes WHERE source_url = ? LIMIT 1")
                .bind(source_url)
                .fetch_optional(&mut *tx)
                .await?;

        if let Some(existing_id) = existing {
            tx.commit().await?;
            let recipe = get(pool, &existing_id).await?;
            return Ok((recipe, false));
        }
    }

    insert(&mut *tx, id, input).await?;
    tags::replace_for_recipe(&mut tx, id, &input.tags).await?;
    autotag::apply(&mut tx, id, input).await?;
    tx.commit().await?;

    let recipe = get(pool, id).await?;
    Ok((recipe, true))
}

/// Inserta la fila de una receta usando el ejecutor dado (pool o transacción).
async fn insert<'e, E>(executor: E, id: &str, input: &RecipeInput) -> AppResult<()>
where
    E: sqlx::Executor<'e, Database = Sqlite>,
{
    let diet_type = input
        .diet_type
        .clone()
        .unwrap_or_else(|| "mediterranean".to_string());
    let servings = input.servings.unwrap_or(2);
    let ingredients = serde_json::to_string(&input.ingredients)
        .map_err(|error| AppError::Internal(anyhow!("serializando ingredientes: {error}")))?;

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
    .execute(executor)
    .await?;

    Ok(())
}

/// Obtiene una receta completa por id (con sus etiquetas); 404 si no existe.
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

    let row = row.ok_or_else(|| AppError::NotFound(format!("receta {id}")))?;
    let tags = tags::for_recipe(pool, id).await?;
    row.into_recipe(tags)
}

/// Reemplaza una receta existente y devuelve su estado final.
///
/// Usa `BEGIN IMMEDIATE` (escritura read-then-write) para comprobar la
/// existencia y reemplazar la fila de forma atómica. Reutiliza los mismos
/// valores por defecto que [`create`] (`diet_type` → `mediterranean`,
/// `servings` → 2). Devuelve 404 si la receta no existe, sin modificar nada.
pub async fn update(pool: &SqlitePool, id: &str, input: &RecipeInput) -> AppResult<Recipe> {
    let diet_type = input
        .diet_type
        .clone()
        .unwrap_or_else(|| "mediterranean".to_string());
    let servings = input.servings.unwrap_or(2);
    let ingredients = serde_json::to_string(&input.ingredients)
        .map_err(|error| AppError::Internal(anyhow!("serializando ingredientes: {error}")))?;

    // Escritura read-then-write: BEGIN IMMEDIATE evita SQLITE_BUSY_SNAPSHOT en WAL.
    let mut tx = pool.begin_with("BEGIN IMMEDIATE").await?;

    let exists: Option<String> = sqlx::query_scalar("SELECT id FROM recipes WHERE id = ?")
        .bind(id)
        .fetch_optional(&mut *tx)
        .await?;
    if exists.is_none() {
        return Err(AppError::NotFound(format!("receta {id}")));
    }

    sqlx::query(
        "UPDATE recipes SET name = ?, category = ?, diet_type = ?, prep_time_minutes = ?, \
         servings = ?, ingredients = ?, instructions = ?, source_url = ? WHERE id = ?",
    )
    .bind(&input.name)
    .bind(&input.category)
    .bind(&diet_type)
    .bind(input.prep_time_minutes)
    .bind(servings)
    .bind(ingredients)
    .bind(&input.instructions)
    .bind(&input.source_url)
    .bind(id)
    .execute(&mut *tx)
    .await?;

    // Reemplaza el conjunto de etiquetas manuales y recalcula las automáticas
    // dentro de la misma transacción.
    tags::replace_for_recipe(&mut tx, id, &input.tags).await?;
    autotag::apply(&mut tx, id, input).await?;

    tx.commit().await?;

    get(pool, id).await
}

/// Recalcula las etiquetas automáticas de una receta existente.
///
/// Conserva las etiquetas `manual` y las supresiones registradas; devuelve la
/// `Recipe` actualizada. 404 si la receta no existe.
pub async fn recalculate_autotags(pool: &SqlitePool, id: &str) -> AppResult<Recipe> {
    // Escritura read-then-write: BEGIN IMMEDIATE evita SQLITE_BUSY_SNAPSHOT en WAL.
    let mut tx = pool.begin_with("BEGIN IMMEDIATE").await?;

    let row: Option<(String, String, Option<String>)> = sqlx::query_as(
        "SELECT name, CAST(ingredients AS TEXT) AS ingredients, instructions \
         FROM recipes WHERE id = ?",
    )
    .bind(id)
    .fetch_optional(&mut *tx)
    .await?;

    let (name, ingredients_json, instructions) =
        row.ok_or_else(|| AppError::NotFound(format!("receta {id}")))?;
    let ingredients: Vec<IngredientInput> = serde_json::from_str(&ingredients_json)
        .map_err(|error| AppError::Internal(anyhow!("ingredientes JSON inválidos: {error}")))?;

    autotag::recalculate(
        &mut tx,
        id,
        &ingredients,
        &name,
        instructions.as_deref(),
        true,
    )
    .await?;
    tx.commit().await?;

    get(pool, id).await
}

/// Elimina una receta; 404 si no existe y 409 si está referenciada por el plan.
///
/// El borrado es atómico (`BEGIN IMMEDIATE`). Los ingredientes **no** son una
/// tabla propia (viven en la columna `ingredients JSON`), así que basta con
/// borrar la fila; los triggers FTS (`recipes_ad`) limpian el índice. Si la
/// receta está referenciada por una entrada de `meal_plan_entries`
/// (`ON DELETE RESTRICT`), se devuelve [`AppError::Conflict`] (409) en lugar de
/// un error de base de datos.
pub async fn delete(pool: &SqlitePool, id: &str) -> AppResult<()> {
    // Escritura read-then-write: BEGIN IMMEDIATE evita SQLITE_BUSY_SNAPSHOT en WAL.
    let mut tx = pool.begin_with("BEGIN IMMEDIATE").await?;

    let exists: Option<String> = sqlx::query_scalar("SELECT id FROM recipes WHERE id = ?")
        .bind(id)
        .fetch_optional(&mut *tx)
        .await?;
    if exists.is_none() {
        return Err(AppError::NotFound(format!("receta {id}")));
    }

    let result = sqlx::query("DELETE FROM recipes WHERE id = ?")
        .bind(id)
        .execute(&mut *tx)
        .await;

    match result {
        Ok(_) => tx.commit().await?,
        // La FK del plan (`ON DELETE RESTRICT`) impide el borrado: se traduce a
        // un 409 controlado (la transacción se revierte al soltar `tx`).
        Err(error) if is_foreign_key_violation(&error) => {
            return Err(AppError::Conflict(format!(
                "la receta {id} está en uso en el plan semanal"
            )));
        }
        Err(error) => return Err(error.into()),
    }

    Ok(())
}

/// Indica si el error es una violación de clave foránea.
///
/// SQLite implementa las FK con triggers internos, por lo que un
/// `ON DELETE RESTRICT` se reporta como `SQLITE_CONSTRAINT_TRIGGER` (código
/// extendido **1811**) en lugar de `SQLITE_CONSTRAINT_FOREIGNKEY` (**787**).
/// `sqlx` mapea el 1811 a `ErrorKind::Other`, así que `is_foreign_key_violation()`
/// no lo detecta; por eso se comprueban también el código extendido y el
/// mensaje canónico.
fn is_foreign_key_violation(error: &sqlx::Error) -> bool {
    match error {
        sqlx::Error::Database(db) => {
            db.is_foreign_key_violation()
                || matches!(db.code().as_deref(), Some("787") | Some("1811"))
                || db.message().contains("FOREIGN KEY constraint failed")
        }
        _ => false,
    }
}
