//! Repositorio de la planificación de comidas (SQLite vía `sqlx`).

use sqlx::SqlitePool;

use crate::domain::date::canonical_iso_date;
use crate::error::{AppError, AppResult};
use crate::models::meal_plan::{MealPlanEntryView, MealPlanInput, MealPlanRow};

/// Orden lógico de los tipos de comida dentro de un día.
const MEAL_TYPE_ORDER: &str = "CASE e.meal_type \
     WHEN 'breakfast' THEN 0 \
     WHEN 'lunch' THEN 1 \
     WHEN 'snack' THEN 2 \
     WHEN 'dinner' THEN 3 \
     ELSE 4 END";

/// Proyección común de una entrada con su receta.
const SELECT_ENTRY: &str = "SELECT e.id, CAST(e.date AS TEXT) AS date, e.meal_type, e.role, \
     COALESCE(e.servings, 2) AS servings, \
     r.id AS recipe_id, r.name AS recipe_name, r.category AS recipe_category, \
     r.diet_type AS recipe_diet_type, r.prep_time_minutes AS recipe_prep_time_minutes, \
     COALESCE(r.servings, 2) AS recipe_servings, \
     CAST(r.created_at AS TEXT) AS recipe_created_at \
     FROM meal_plan_entries e JOIN recipes r ON r.id = e.recipe_id";

/// Lista las entradas del plan en un rango de fechas (inclusive).
///
/// Ordena por fecha, luego por tipo de comida (desayuno, almuerzo, merienda,
/// cena) y, finalmente, por fecha de creación.
pub async fn list(
    pool: &SqlitePool,
    start_date: &str,
    end_date: &str,
) -> AppResult<Vec<MealPlanEntryView>> {
    let sql = format!(
        "{SELECT_ENTRY} WHERE e.date >= ? AND e.date <= ? \
         ORDER BY e.date ASC, {MEAL_TYPE_ORDER} ASC, CAST(e.created_at AS TEXT) ASC"
    );

    let rows: Vec<MealPlanRow> = sqlx::query_as(sqlx::AssertSqlSafe(sql))
        .bind(start_date)
        .bind(end_date)
        .fetch_all(pool)
        .await?;

    Ok(rows.into_iter().map(MealPlanRow::into_view).collect())
}

/// Crea una entrada del plan, validando que la receta referenciada exista.
pub async fn create(
    pool: &SqlitePool,
    id: &str,
    input: &MealPlanInput,
) -> AppResult<MealPlanEntryView> {
    let recipe_exists: Option<String> = sqlx::query_scalar("SELECT id FROM recipes WHERE id = ?")
        .bind(&input.recipe_id)
        .fetch_optional(pool)
        .await?;

    if recipe_exists.is_none() {
        return Err(AppError::Validation("receta no encontrada".to_string()));
    }

    // La fecha se persiste siempre en su forma canónica `YYYY-MM-DD` para no
    // romper las comparaciones de rango ni el orden lexicográfico.
    let date = canonical_iso_date(&input.date)
        .ok_or_else(|| AppError::Validation(format!("fecha inválida: {}", input.date)))?;

    let servings = input.servings.unwrap_or(2);

    sqlx::query(
        "INSERT INTO meal_plan_entries (id, date, meal_type, recipe_id, role, servings) \
         VALUES (?, ?, ?, ?, ?, ?)",
    )
    .bind(id)
    .bind(&date)
    .bind(&input.meal_type)
    .bind(&input.recipe_id)
    .bind(&input.role)
    .bind(servings)
    .execute(pool)
    .await?;

    get(pool, id).await
}

/// Obtiene una entrada por id; 404 si no existe.
pub async fn get(pool: &SqlitePool, id: &str) -> AppResult<MealPlanEntryView> {
    let sql = format!("{SELECT_ENTRY} WHERE e.id = ?");
    let row: Option<MealPlanRow> = sqlx::query_as(sqlx::AssertSqlSafe(sql))
        .bind(id)
        .fetch_optional(pool)
        .await?;

    match row {
        Some(row) => Ok(row.into_view()),
        None => Err(AppError::NotFound(format!("entrada del plan {id}"))),
    }
}

/// Elimina una entrada del plan; 404 si no existía.
pub async fn delete(pool: &SqlitePool, id: &str) -> AppResult<()> {
    let result = sqlx::query("DELETE FROM meal_plan_entries WHERE id = ?")
        .bind(id)
        .execute(pool)
        .await?;

    if result.rows_affected() == 0 {
        return Err(AppError::NotFound(format!("entrada del plan {id}")));
    }

    Ok(())
}
