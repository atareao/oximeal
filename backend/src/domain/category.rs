//! Categorización heurística por pasillo.

const VERDULERIA: &str = "Verdulería";
const PESCADERIA: &str = "Pescadería";
const CARNICERIA: &str = "Carnicería";
const REFRIGERADOS: &str = "Refrigerados";
const PANADERIA: &str = "Panadería";
const BEBIDAS: &str = "Bebidas";
const DESPENSA: &str = "Despensa";
const OTROS: &str = "Otros";

/// Orden canónico de pasillos de la lista de la compra.
///
/// Fuente única de verdad: tanto el ranking de consolidación
/// (`domain::consolidate::category_rank`) como el agrupado del repositorio
/// (`repo::shopping_list`) derivan de este orden. `"Otros"` va siempre al final.
pub const CATEGORY_ORDER: [&str; 8] = [
    VERDULERIA,
    PESCADERIA,
    CARNICERIA,
    REFRIGERADOS,
    PANADERIA,
    BEBIDAS,
    DESPENSA,
    OTROS,
];

/// Tabla ordenada de reglas `keyword → categoría`. La primera coincidencia gana.
///
/// El orden importa: las categorías más específicas (y las excepciones como
/// "tomate frito") van antes que las genéricas.
static RULES: &[(&str, &str)] = &[
    // Excepciones específicas antes que su palabra base.
    ("tomate frito", DESPENSA),
    ("harina de trigo", PANADERIA),
    // Verdulería / Frutería
    ("tomate", VERDULERIA),
    ("cebolla", VERDULERIA),
    ("cebollino", VERDULERIA),
    ("ajo", VERDULERIA),
    ("patata", VERDULERIA),
    ("zanahoria", VERDULERIA),
    ("lechuga", VERDULERIA),
    ("canonigo", VERDULERIA),
    ("pimiento", VERDULERIA),
    ("calabacin", VERDULERIA),
    ("calabaza", VERDULERIA),
    ("berenjena", VERDULERIA),
    ("pepino", VERDULERIA),
    ("limon", VERDULERIA),
    ("manzana", VERDULERIA),
    ("platano", VERDULERIA),
    ("naranja", VERDULERIA),
    ("pera", VERDULERIA),
    ("uva", VERDULERIA),
    ("sandia", VERDULERIA),
    ("melon", VERDULERIA),
    ("perejil", VERDULERIA),
    ("espinaca", VERDULERIA),
    ("acelga", VERDULERIA),
    ("judia", VERDULERIA),
    ("guisante", VERDULERIA),
    ("champinon", VERDULERIA),
    ("seta", VERDULERIA),
    ("brocoli", VERDULERIA),
    ("coliflor", VERDULERIA),
    ("alcachofa", VERDULERIA),
    ("apio", VERDULERIA),
    ("puerro", VERDULERIA),
    ("remolacha", VERDULERIA),
    ("aguacate", VERDULERIA),
    ("kiwi", VERDULERIA),
    ("fresa", VERDULERIA),
    ("cereza", VERDULERIA),
    // Pescadería
    ("lubina", PESCADERIA),
    ("merluza", PESCADERIA),
    ("salmon", PESCADERIA),
    ("gamba", PESCADERIA),
    ("langostino", PESCADERIA),
    ("atun", PESCADERIA),
    ("pulpo", PESCADERIA),
    ("boqueron", PESCADERIA),
    ("dorada", PESCADERIA),
    ("bacalao", PESCADERIA),
    ("sardina", PESCADERIA),
    ("calamar", PESCADERIA),
    ("mejillon", PESCADERIA),
    ("almeja", PESCADERIA),
    ("trucha", PESCADERIA),
    ("rape", PESCADERIA),
    ("besugo", PESCADERIA),
    ("jurel", PESCADERIA),
    ("rodaballo", PESCADERIA),
    ("pescadilla", PESCADERIA),
    // Carnicería
    ("pollo", CARNICERIA),
    ("ternera", CARNICERIA),
    ("cerdo", CARNICERIA),
    ("cordero", CARNICERIA),
    ("pavo", CARNICERIA),
    ("chorizo", CARNICERIA),
    ("jamon", CARNICERIA),
    ("carne picada", CARNICERIA),
    ("carne", CARNICERIA),
    ("morcilla", CARNICERIA),
    ("salchicha", CARNICERIA),
    ("costilla", CARNICERIA),
    ("lomo", CARNICERIA),
    ("solomillo", CARNICERIA),
    ("bacon", CARNICERIA),
    ("panceta", CARNICERIA),
    ("chuleta", CARNICERIA),
    ("higado", CARNICERIA),
    ("conejo", CARNICERIA),
    ("pato", CARNICERIA),
    ("butifarra", CARNICERIA),
    // Refrigerados
    ("leche", REFRIGERADOS),
    ("huevo", REFRIGERADOS),
    ("yogur", REFRIGERADOS),
    ("queso", REFRIGERADOS),
    ("mantequilla", REFRIGERADOS),
    ("margarina", REFRIGERADOS),
    ("nata", REFRIGERADOS),
    ("requeson", REFRIGERADOS),
    ("cuajada", REFRIGERADOS),
    ("kefir", REFRIGERADOS),
    ("tofu", REFRIGERADOS),
    ("tempeh", REFRIGERADOS),
    // Panadería
    ("pan", PANADERIA),
    ("baguette", PANADERIA),
    ("bollo", PANADERIA),
    ("croissant", PANADERIA),
    ("magdalena", PANADERIA),
    ("bizcocho", PANADERIA),
    ("tostada", PANADERIA),
    ("mollete", PANADERIA),
    ("panecillo", PANADERIA),
    // Bebidas
    ("agua", BEBIDAS),
    ("vino", BEBIDAS),
    ("cerveza", BEBIDAS),
    ("zumo", BEBIDAS),
    ("refresco", BEBIDAS),
    ("sidra", BEBIDAS),
    ("cava", BEBIDAS),
    ("champan", BEBIDAS),
    ("licor", BEBIDAS),
    ("ginebra", BEBIDAS),
    ("whisky", BEBIDAS),
    ("cafe", BEBIDAS),
    ("infusion", BEBIDAS),
    ("batido", BEBIDAS),
    // Despensa
    ("arroz", DESPENSA),
    ("pasta", DESPENSA),
    ("lenteja", DESPENSA),
    ("garbanzo", DESPENSA),
    ("alubia", DESPENSA),
    ("aceite", DESPENSA),
    ("sal", DESPENSA),
    ("harina", DESPENSA),
    ("azucar", DESPENSA),
    ("conserva", DESPENSA),
    ("especia", DESPENSA),
    ("oregano", DESPENSA),
    ("pimienta", DESPENSA),
    ("pimenton", DESPENSA),
    ("comino", DESPENSA),
    ("canela", DESPENSA),
    ("vinagre", DESPENSA),
    ("miel", DESPENSA),
    ("levadura", DESPENSA),
    ("cacao", DESPENSA),
    ("chocolate", DESPENSA),
    ("galleta", DESPENSA),
    ("cereal", DESPENSA),
    ("avena", DESPENSA),
    ("quinoa", DESPENSA),
    ("fideo", DESPENSA),
    ("macarron", DESPENSA),
    ("espagueti", DESPENSA),
];

/// Asigna una categoría de pasillo a un nombre ya normalizado.
///
/// La primera regla cuyo keyword aparezca como subcadena del nombre gana; si
/// ninguna coincide, devuelve `"Otros"`.
pub fn categorize(name_normalized: &str) -> &'static str {
    for &(keyword, category) in RULES {
        if name_normalized.contains(keyword) {
            return category;
        }
    }
    OTROS
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn pescaderia() {
        assert_eq!(categorize("merluza"), "Pescadería");
    }

    #[test]
    fn verduleria() {
        assert_eq!(categorize("tomate"), "Verdulería");
    }

    #[test]
    fn despensa() {
        assert_eq!(categorize("arroz"), "Despensa");
    }

    #[test]
    fn refrigerados() {
        assert_eq!(categorize("queso"), "Refrigerados");
    }

    #[test]
    fn desconocido_es_otros() {
        assert_eq!(categorize("xyzzy"), "Otros");
    }

    #[test]
    fn excepcion_tomate_frito() {
        assert_eq!(categorize("tomate frito"), "Despensa");
    }

    #[test]
    fn frutas_van_a_verduleria() {
        let frutas = [
            "melocoton",
            "nectarina",
            "albaricoque",
            "ciruela",
            "pina",
            "mango",
            "aguacate",
            "granada",
            "higo",
            "frambuesa",
            "arandano",
            "mora",
            "mandarina",
            "pomelo",
            "coco",
            "datil",
            "caqui",
            "paraguayo",
        ];
        for fruta in frutas {
            assert_eq!(categorize(fruta), "Verdulería", "fruta: {fruta}");
        }
    }

    #[test]
    fn verduras_van_a_verduleria() {
        let verduras = [
            "espinaca",
            "acelga",
            "lechuga",
            "cebolla",
            "ajo",
            "puerro",
            "zanahoria",
            "calabacin",
            "calabaza",
            "berenjena",
            "pimiento",
            "pepino",
            "patata",
            "boniato",
            "judia",
            "guisante",
            "alcachofa",
            "brocoli",
            "coliflor",
            "repollo",
            "apio",
            "remolacha",
            "rabano",
            "seta",
            "champinon",
            "maiz",
            "esparrago",
        ];
        for verdura in verduras {
            assert_eq!(categorize(verdura), "Verdulería", "verdura: {verdura}");
        }
    }

    #[test]
    fn legumbres_secas_van_a_despensa() {
        assert_eq!(categorize("garbanzo"), "Despensa");
        assert_eq!(categorize("lenteja"), "Despensa");
    }

    #[test]
    fn colisiones_ganan_la_regla_especifica() {
        assert_eq!(categorize("morcilla"), "Carnicería");
        assert_eq!(categorize("higado"), "Carnicería");
        assert_eq!(categorize("limonada"), "Bebidas");
        assert_eq!(categorize("manzanilla"), "Despensa");
    }

    #[test]
    fn orden_de_categorias_es_unico_y_termina_en_otros() {
        assert_eq!(CATEGORY_ORDER.len(), 8);
        assert_eq!(CATEGORY_ORDER.last(), Some(&"Otros"));
        for (index, category) in CATEGORY_ORDER.iter().enumerate() {
            assert!(
                !CATEGORY_ORDER[index + 1..].contains(category),
                "categoría duplicada: {category}"
            );
        }
    }
}
