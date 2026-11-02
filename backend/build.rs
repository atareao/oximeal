//! Script de build del crate `oximeal-backend`.
//!
//! Existe por una única razón: la feature `embed-ui` usa `rust-embed` con
//! `#[folder = "$CARGO_MANIFEST_DIR/../frontend/dist/"]`, y `rust-embed` falla
//! en tiempo de compilación si esa carpeta no existe. En un clon limpio (sin
//! `frontend/dist`, que está en `.gitignore`) `cargo test --features embed-ui`
//! no compilaría.
//!
//! Para evitarlo, **cuando la feature `embed-ui` está activa** y `frontend/dist`
//! no existe, creamos un `index.html` placeholder mínimo. Así el crate compila
//! siempre, con o sin frontend compilado, y `just ui-build` lo sobrescribe con
//! la UI real.
//!
//! El placeholder solo se crea con `embed-ui`: sin la feature, el macro
//! `#[folder]` ni siquiera se expande, así que no hay motivo para escribir en
//! el árbol de fuentes (`frontend/dist`). Cargo exporta `CARGO_FEATURE_<NAME>`
//! (en mayúsculas y con `-` → `_`) por cada feature activa; de ahí
//! `CARGO_FEATURE_EMBED_UI`.

use std::path::PathBuf;

/// Contenido mínimo del placeholder cuando no hay frontend compilado.
const PLACEHOLDER: &str = concat!(
    "<!doctype html>\n",
    "<html lang=\"es\">\n",
    "<head><meta charset=\"utf-8\"><title>OxiMeal</title></head>\n",
    "<body><p>UI no compilada. Ejecuta `just ui-build`.</p></body>\n",
    "</html>\n",
);

fn main() {
    let manifest_dir = match std::env::var("CARGO_MANIFEST_DIR") {
        Ok(dir) => dir,
        // Sin la variable no podemos localizar `dist`; no hay nada que hacer.
        Err(_) => return,
    };

    let dist_dir = PathBuf::from(manifest_dir)
        .join("..")
        .join("frontend")
        .join("dist");

    // Cargo define `CARGO_FEATURE_<NAME>` si la feature está activa.
    let embed_ui = std::env::var_os("CARGO_FEATURE_EMBED_UI").is_some();

    if embed_ui && !dist_dir.exists() {
        if let Err(error) = std::fs::create_dir_all(&dist_dir) {
            println!(
                "cargo:warning=no se pudo crear {}: {error}",
                dist_dir.display()
            );
            return;
        }

        let index = dist_dir.join("index.html");
        if let Err(error) = std::fs::write(&index, PLACEHOLDER) {
            println!(
                "cargo:warning=no se pudo escribir {}: {error}",
                index.display()
            );
        } else {
            println!(
                "cargo:warning=frontend/dist no existe: se ha creado un index.html placeholder. \
                 Ejecuta `just ui-build` para compilar la UI real."
            );
        }
    }

    // Recompilar si el frontend cambia (o si aparece/desaparece `dist`).
    println!("cargo:rerun-if-changed={}", dist_dir.display());
}
