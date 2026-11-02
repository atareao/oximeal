//! Normalización de nombres de ingredientes.

/// Elimina los diacríticos del español (y de otros latinos comunes).
///
/// Se aplica sobre texto ya en minúsculas; se implementa con un mapeo explícito
/// para no añadir dependencias de normalización Unicode al stack declarado.
pub(crate) fn deaccent(input: &str) -> String {
    input.chars().map(fold_char).collect()
}

fn fold_char(c: char) -> char {
    match c {
        'á' | 'à' | 'â' | 'ä' | 'ã' | 'å' => 'a',
        'é' | 'è' | 'ê' | 'ë' => 'e',
        'í' | 'ì' | 'î' | 'ï' => 'i',
        'ó' | 'ò' | 'ô' | 'ö' | 'õ' => 'o',
        'ú' | 'ù' | 'û' | 'ü' => 'u',
        'ñ' => 'n',
        'ç' => 'c',
        'ý' | 'ÿ' => 'y',
        other => other,
    }
}

const STOPWORDS: &[&str] = &[
    "el", "la", "los", "las", "un", "una", "unos", "unas", "de", "del",
];

/// Aplica una singularización básica a un nombre de una sola palabra.
///
/// - `...ces` → `...z` (nueces → nuez).
/// - termina en `s`, longitud > 3 y la letra anterior es vocal → se quita la `s`.
fn singularize(word: &str) -> String {
    if word.is_empty() {
        return String::new();
    }
    if let Some(stem) = word.strip_suffix("ces")
        && !stem.is_empty()
    {
        return format!("{stem}z");
    }
    let chars: Vec<char> = word.chars().collect();
    if chars.len() > 3
        && word.ends_with('s')
        && matches!(chars[chars.len() - 2], 'a' | 'e' | 'i' | 'o' | 'u')
    {
        return word[..word.len() - 1].to_string();
    }
    word.to_string()
}

/// Normaliza un nombre de ingrediente a una clave comparable.
///
/// Pasos: minúsculas → sin diacríticos → colapso de espacios → eliminación de
/// artículos/preposiciones iniciales → singularización básica. La singularización
/// solo se aplica a nombres de una sola palabra para no alterar locuciones como
/// "aceite de oliva".
pub fn normalize_name(name: &str) -> String {
    let lowered = name.to_lowercase();
    let folded = deaccent(&lowered);
    let collapsed = folded.split_whitespace().collect::<Vec<_>>().join(" ");

    let tokens: Vec<&str> = collapsed.split(' ').collect();
    let mut start = 0;
    while start + 1 < tokens.len() && STOPWORDS.contains(&tokens[start]) {
        start += 1;
    }
    let joined = tokens[start..].join(" ");

    if joined.contains(' ') {
        joined
    } else {
        singularize(&joined)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn cebollas_y_la_cebolla_misma_clave() {
        assert_eq!(normalize_name("Cebollas"), normalize_name("la cebolla"));
    }

    #[test]
    fn singular_terminado_en_s() {
        assert_eq!(normalize_name("tomates"), "tomate");
        assert_eq!(normalize_name("cebollas"), "cebolla");
        assert_eq!(normalize_name("patatas"), "patata");
        assert_eq!(normalize_name("lentejas"), "lenteja");
        assert_eq!(normalize_name("garbanzos"), "garbanzo");
        assert_eq!(normalize_name("huevos"), "huevo");
    }

    #[test]
    fn plural_en_ces() {
        assert_eq!(normalize_name("nueces"), "nuez");
    }

    #[test]
    fn conserva_articulo_interno() {
        assert_eq!(normalize_name("Aceite de oliva"), "aceite de oliva");
    }

    #[test]
    fn elimina_diacriticos_y_colapsa_espacios() {
        assert_eq!(
            normalize_name("  Plátano   de   Canarias "),
            "platano de canarias"
        );
    }

    #[test]
    fn elimina_articulo_inicial_encadenado() {
        assert_eq!(normalize_name("de la cebolla"), "cebolla");
    }
}
