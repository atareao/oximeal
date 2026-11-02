//! Normalización de fechas del dominio (sin depender de la capa HTTP).

use chrono::NaiveDate;

/// Parsea una fecha `YYYY-MM-DD`.
///
/// `chrono` acepta componentes sin rellenar (p. ej. `2026-1-2`), por lo que el
/// valor original **no** debe persistirse tal cual: hay que canonicalizarlo.
pub fn parse_iso_date(value: &str) -> Option<NaiveDate> {
    NaiveDate::parse_from_str(value, "%Y-%m-%d").ok()
}

/// Devuelve la fecha en su forma canónica `YYYY-MM-DD` (componentes rellenados
/// a dos dígitos). Devuelve `None` si el valor no es una fecha válida.
pub fn canonical_iso_date(value: &str) -> Option<String> {
    parse_iso_date(value).map(|date| date.format("%Y-%m-%d").to_string())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn canonicaliza_componentes_sin_rellenar() {
        assert_eq!(
            canonical_iso_date("2026-1-2").as_deref(),
            Some("2026-01-02")
        );
        assert_eq!(
            canonical_iso_date("2026-02-02").as_deref(),
            Some("2026-02-02")
        );
    }

    #[test]
    fn rechaza_fechas_invalidas() {
        assert_eq!(canonical_iso_date("2026-13-01"), None);
        assert_eq!(canonical_iso_date("no-es-fecha"), None);
    }
}
