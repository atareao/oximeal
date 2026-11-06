//! Repositorio de etiquetas (SQLite vía `sqlx`).
//!
//! La identidad de una etiqueta es su `slug` normalizado ([`slugify`]), que es
//! `UNIQUE` en la base de datos. El alta es idempotente por `slug`, y las
//! operaciones de escritura usan `BEGIN IMMEDIATE` para evitar
//! `SQLITE_BUSY_SNAPSHOT` en WAL y cerrar la ventana de carrera entre el
//! `SELECT` y el `INSERT`.

use std::collections::{HashMap, HashSet};

use anyhow::anyhow;
use sqlx::{QueryBuilder, Sqlite, SqliteConnection, SqlitePool};
use uuid::Uuid;

use crate::error::{AppError, AppResult};
use crate::models::tag::{RecipeTag, Tag, TagRow, display_name, slugify};
use crate::services::autotag;

/// Proyección de una etiqueta con su recuento de recetas asociadas.
const SELECT_TAG_WITH_COUNT: &str = "SELECT t.id, t.name, t.slug, \
     (SELECT COUNT(*) FROM recipe_tags rt WHERE rt.tag_id = t.id) AS recipe_count \
     FROM tags t";

/// Lista todas las etiquetas con su recuento de recetas, ordenadas por nombre.
///
/// Las etiquetas sin recetas asociadas aparecen con `recipe_count = 0`.
pub async fn list(pool: &SqlitePool) -> AppResult<Vec<Tag>> {
    let sql = format!("{SELECT_TAG_WITH_COUNT} ORDER BY t.name ASC, t.id ASC");
    let rows: Vec<TagRow> = sqlx::query_as(sqlx::AssertSqlSafe(sql))
        .fetch_all(pool)
        .await?;
    Ok(rows.into_iter().map(Tag::from).collect())
}

/// Crea una etiqueta de forma idempotente por `slug`.
///
/// Devuelve `(tag, created)`: `created = true` si se acaba de crear, o `false`
/// si ya existía una con el mismo `slug` (comparación insensible a mayúsculas y
/// espacios), en cuyo caso devuelve la existente sin modificarla.
pub async fn create(pool: &SqlitePool, raw_name: &str) -> AppResult<(Tag, bool)> {
    // Se aplica la misma canonicalización que `replace_for_recipe`/`resolve`:
    // una etiqueta del catálogo automático («Sin gluten») comparte el `slug`
    // canónico (`sin-gluten`) y no se duplica.
    let (name, slug) = resolve(raw_name);

    // Escritura read-then-write: BEGIN IMMEDIATE evita SQLITE_BUSY_SNAPSHOT en WAL.
    let mut tx = pool.begin_with("BEGIN IMMEDIATE").await?;

    if let Some(existing_id) = find_id_by_slug(&mut *tx, &slug).await? {
        let tag = fetch_with_count(&mut *tx, &existing_id)
            .await?
            .ok_or_else(|| AppError::Internal(anyhow!("la etiqueta {slug} desapareció")))?;
        tx.commit().await?;
        return Ok((tag, false));
    }

    let id = Uuid::new_v4().to_string();
    sqlx::query("INSERT INTO tags (id, name, slug) VALUES (?, ?, ?)")
        .bind(&id)
        .bind(&name)
        .bind(&slug)
        .execute(&mut *tx)
        .await?;
    tx.commit().await?;

    Ok((
        Tag {
            id,
            name,
            slug,
            recipe_count: 0,
        },
        true,
    ))
}

/// Elimina una etiqueta (404 si no existe).
///
/// La FK `recipe_tags.tag_id ON DELETE CASCADE` borra sus asociaciones; las
/// recetas afectadas permanecen intactas. El borrado es atómico.
pub async fn delete(pool: &SqlitePool, id: &str) -> AppResult<()> {
    let mut tx = pool.begin_with("BEGIN IMMEDIATE").await?;

    let exists: Option<String> = sqlx::query_scalar("SELECT id FROM tags WHERE id = ?")
        .bind(id)
        .fetch_optional(&mut *tx)
        .await?;
    if exists.is_none() {
        return Err(AppError::NotFound(format!("etiqueta {id}")));
    }

    sqlx::query("DELETE FROM tags WHERE id = ?")
        .bind(id)
        .execute(&mut *tx)
        .await?;
    tx.commit().await?;

    Ok(())
}

/// Reemplaza el conjunto de etiquetas **manuales** de una receta.
///
/// Se ejecuta dentro de la misma transacción `BEGIN IMMEDIATE` que la escritura
/// de la receta. Borra solo las asociaciones `manual` (las `auto` las gestiona
/// [`crate::services::autotag`]), crea al vuelo las etiquetas que no existan
/// (comparando por `slug`) y **deduplica** por `slug`. Una etiqueta del catálogo
/// automático se resuelve a su `slug` canónico (p. ej. «Sin gluten» →
/// `sin-gluten`) y, al ser manual, elimina su supresión (la manual gana).
pub(crate) async fn replace_for_recipe(
    conn: &mut SqliteConnection,
    recipe_id: &str,
    names: &[String],
) -> AppResult<()> {
    sqlx::query("DELETE FROM recipe_tags WHERE recipe_id = ? AND source = 'manual'")
        .bind(recipe_id)
        .execute(&mut *conn)
        .await?;

    let mut seen: HashSet<String> = HashSet::new();
    for raw in names {
        let (name, slug) = resolve(raw);
        if slug.is_empty() || !seen.insert(slug.clone()) {
            continue;
        }

        let tag_id = upsert(&mut *conn, &name, &slug).await?;
        sqlx::query(
            "INSERT INTO recipe_tags (recipe_id, tag_id, source) VALUES (?, ?, 'manual') \
             ON CONFLICT(recipe_id, tag_id) DO UPDATE SET source = 'manual'",
        )
        .bind(recipe_id)
        .bind(&tag_id)
        .execute(&mut *conn)
        .await?;

        // Añadir manualmente una etiqueta suprimida borra su supresión.
        sqlx::query("DELETE FROM recipe_tag_suppressions WHERE recipe_id = ? AND tag_id = ?")
            .bind(recipe_id)
            .bind(&tag_id)
            .execute(&mut *conn)
            .await?;
    }

    Ok(())
}

/// Reemplaza el conjunto de supresiones automáticas de una receta.
///
/// Resuelve cada nombre o `slug` a una etiqueta (creándola si no existe) y
/// registra la supresión. Una etiqueta ya fijada como `manual` no se suprime.
pub(crate) async fn replace_suppressions(
    conn: &mut SqliteConnection,
    recipe_id: &str,
    names: &[String],
) -> AppResult<()> {
    sqlx::query("DELETE FROM recipe_tag_suppressions WHERE recipe_id = ?")
        .bind(recipe_id)
        .execute(&mut *conn)
        .await?;

    let mut seen: HashSet<String> = HashSet::new();
    for raw in names {
        let (name, slug) = resolve(raw);
        if slug.is_empty() || !seen.insert(slug.clone()) {
            continue;
        }

        let tag_id = upsert(&mut *conn, &name, &slug).await?;

        let manual: Option<String> = sqlx::query_scalar(
            "SELECT source FROM recipe_tags \
             WHERE recipe_id = ? AND tag_id = ? AND source = 'manual'",
        )
        .bind(recipe_id)
        .bind(&tag_id)
        .fetch_optional(&mut *conn)
        .await?;
        if manual.is_some() {
            continue;
        }

        sqlx::query(
            "INSERT INTO recipe_tag_suppressions (recipe_id, tag_id) VALUES (?, ?) \
             ON CONFLICT DO NOTHING",
        )
        .bind(recipe_id)
        .bind(&tag_id)
        .execute(&mut *conn)
        .await?;
    }

    Ok(())
}

/// Resuelve un nombre de etiqueta a `(nombre, slug)`.
///
/// Si corresponde a una etiqueta del catálogo automático, usa su `slug` y nombre
/// canónicos; en caso contrario, los deriva con [`display_name`] y [`slugify`].
fn resolve(raw: &str) -> (String, String) {
    match autotag::canonical_auto_slug(raw) {
        Some(slug) => {
            let name = autotag::name_for_slug(slug).unwrap_or(slug).to_string();
            (name, slug.to_string())
        }
        None => (display_name(raw), slugify(raw)),
    }
}

/// Inserta (si falta) una etiqueta del catálogo automático y devuelve su id.
pub(crate) async fn upsert_auto(conn: &mut SqliteConnection, slug: &str) -> AppResult<String> {
    let name = autotag::name_for_slug(slug).unwrap_or(slug);
    upsert(conn, name, slug).await
}

/// Etiquetas de una receta, ordenadas por nombre (sin recuento).
pub async fn for_recipe(pool: &SqlitePool, recipe_id: &str) -> AppResult<Vec<RecipeTag>> {
    let rows: Vec<(String, String, String, String)> = sqlx::query_as(
        "SELECT t.id, t.name, t.slug, rt.source FROM recipe_tags rt \
         JOIN tags t ON t.id = rt.tag_id WHERE rt.recipe_id = ? \
         ORDER BY t.name ASC, t.id ASC",
    )
    .bind(recipe_id)
    .fetch_all(pool)
    .await?;

    Ok(rows
        .into_iter()
        .map(|(id, name, slug, source)| RecipeTag {
            id,
            name,
            slug,
            source,
        })
        .collect())
}

/// Etiquetas de varias recetas, agrupadas por `recipe_id`.
///
/// Evita el patrón N+1 al poblar listados: una sola consulta `IN (...)`.
pub async fn for_recipes(
    pool: &SqlitePool,
    recipe_ids: &[String],
) -> AppResult<HashMap<String, Vec<RecipeTag>>> {
    let mut grouped: HashMap<String, Vec<RecipeTag>> = HashMap::new();
    if recipe_ids.is_empty() {
        return Ok(grouped);
    }

    let mut query = QueryBuilder::<Sqlite>::new(
        "SELECT rt.recipe_id, t.id, t.name, t.slug, rt.source FROM recipe_tags rt \
         JOIN tags t ON t.id = rt.tag_id WHERE rt.recipe_id IN (",
    );
    {
        let mut separated = query.separated(", ");
        for id in recipe_ids {
            separated.push_bind(id);
        }
    }
    query.push(") ORDER BY t.name ASC, t.id ASC");

    let rows: Vec<(String, String, String, String, String)> =
        query.build_query_as().fetch_all(pool).await?;
    for (recipe_id, id, name, slug, source) in rows {
        grouped.entry(recipe_id).or_default().push(RecipeTag {
            id,
            name,
            slug,
            source,
        });
    }

    Ok(grouped)
}

/// Inserta la etiqueta si su `slug` no existe y devuelve su id.
///
/// `ON CONFLICT(slug) DO NOTHING` respeta el nombre canónico ya almacenado.
async fn upsert(conn: &mut SqliteConnection, name: &str, slug: &str) -> AppResult<String> {
    sqlx::query("INSERT INTO tags (id, name, slug) VALUES (?, ?, ?) ON CONFLICT(slug) DO NOTHING")
        .bind(Uuid::new_v4().to_string())
        .bind(name)
        .bind(slug)
        .execute(&mut *conn)
        .await?;

    let id: String = sqlx::query_scalar("SELECT id FROM tags WHERE slug = ?")
        .bind(slug)
        .fetch_one(&mut *conn)
        .await?;
    Ok(id)
}

/// Busca el id de la etiqueta con el `slug` dado.
async fn find_id_by_slug<'e, E>(executor: E, slug: &str) -> AppResult<Option<String>>
where
    E: sqlx::Executor<'e, Database = Sqlite>,
{
    let id: Option<String> = sqlx::query_scalar("SELECT id FROM tags WHERE slug = ?")
        .bind(slug)
        .fetch_optional(executor)
        .await?;
    Ok(id)
}

/// Obtiene una etiqueta con su recuento por id.
async fn fetch_with_count<'e, E>(executor: E, id: &str) -> AppResult<Option<Tag>>
where
    E: sqlx::Executor<'e, Database = Sqlite>,
{
    let sql = format!("{SELECT_TAG_WITH_COUNT} WHERE t.id = ?");
    let row: Option<TagRow> = sqlx::query_as(sqlx::AssertSqlSafe(sql))
        .bind(id)
        .fetch_optional(executor)
        .await?;
    Ok(row.map(Tag::from))
}
