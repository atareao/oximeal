//! Comparación de tokens anti-CSRF en tiempo constante.
//!
//! El token anti-CSRF viaja en la cabecera `X-CSRF-Token`. Se compara contra
//! el valor guardado en la sesión server-side sin cortocircuitar ante la
//! primera diferencia, para no filtrar el token byte a byte por temporización.

/// Compara el token esperado con el valor recibido en la cabecera.
///
/// Devuelve `true` solo si ambos son no vacíos y coinciden exactamente. La
/// ausencia de cabecera (`None`) o un valor vacío siempre resulta en `false`.
pub fn verify(expected: &str, header: Option<&str>) -> bool {
    if expected.is_empty() {
        return false;
    }
    match header {
        Some(provided) => constant_time_eq(expected.as_bytes(), provided.as_bytes()),
        None => false,
    }
}

/// Igualdad de bytes en tiempo constante respecto al contenido.
///
/// La longitud sí se filtra (no es información sensible en un CSRF token),
/// pero el contenido se compara acumulando el XOR de todos los bytes sin
/// salir de forma anticipada.
fn constant_time_eq(a: &[u8], b: &[u8]) -> bool {
    if a.len() != b.len() {
        return false;
    }
    let mut diff = 0u8;
    for (x, y) in a.iter().zip(b.iter()) {
        diff |= x ^ y;
    }
    diff == 0
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn accepts_matching_token() {
        assert!(verify("abc123", Some("abc123")));
    }

    #[test]
    fn rejects_different_token() {
        assert!(!verify("abc123", Some("abc124")));
    }

    #[test]
    fn rejects_missing_header() {
        assert!(!verify("abc123", None));
    }

    #[test]
    fn rejects_empty_provided() {
        assert!(!verify("abc123", Some("")));
    }

    #[test]
    fn rejects_empty_expected() {
        assert!(!verify("", Some("abc")));
    }

    #[test]
    fn rejects_different_length() {
        assert!(!verify("abc", Some("abcd")));
    }
}
