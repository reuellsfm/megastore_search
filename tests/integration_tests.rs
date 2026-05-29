//! Testes de integração para o sistema de busca.
//!
//! Diferente dos testes unitários (que ficam em cada módulo), estes
//! testes exercitam o sistema como um todo, simulando uso real.

use megastore_search::{Catalog, Product};

fn build_real_catalog() -> Catalog {
    let mut c = Catalog::new();
    let entries = [
        // Eletrônicos
        (1, "Notebook Gamer Pro 15", "Acer", "Eletrônicos", 599900, 4),
        (2, "Notebook Office 14", "Acer", "Eletrônicos", 299900, 8),
        (3, "Smart TV 55 4K UHD", "Samsung", "Eletrônicos", 289900, 6),
        (4, "Smart TV 50 QLED", "Samsung", "Eletrônicos", 349900, 3),
        (5, "Smartphone Galaxy Pro", "Samsung", "Eletrônicos", 459900, 12),
        // Periféricos
        (10, "Mouse Gamer RGB", "Logitech", "Periféricos", 24900, 50),
        (11, "Teclado Mecânico RGB", "Logitech", "Periféricos", 35000, 20),
        (12, "Headset Gamer 7.1", "HyperX", "Periféricos", 45000, 15),
        (13, "Webcam HD", "Logitech", "Periféricos", 19990, 30),
        // Vestuário
        (20, "Camiseta Básica Algodão", "Hering", "Vestuário", 4990, 100),
        (21, "Calça Jeans Slim", "Levi's", "Vestuário", 19990, 40),
        (22, "Jaqueta de Couro", "Levi's", "Vestuário", 39990, 8),
        // Calçados
        (30, "Tênis Air Max", "Nike", "Calçados", 49990, 18),
        (31, "Tênis Casual", "Adidas", "Calçados", 35990, 22),
    ];
    for (id, name, brand, cat, price, stock) in entries {
        c.add_product(Product::new(id, name, brand, cat, price, stock));
    }
    c
}

#[test]
fn integracao_busca_completa_por_id() {
    let c = build_real_catalog();
    assert_eq!(c.len(), 14);

    let p = c.find_by_id(10).unwrap();
    assert_eq!(p.name, "Mouse Gamer RGB");
    assert_eq!(p.price_formatted(), "R$ 249,00");
}

#[test]
fn integracao_busca_por_categoria_eletronicos() {
    let c = build_real_catalog();
    let eletro = c.find_by_category("Eletrônicos");
    assert_eq!(eletro.len(), 5);
}

#[test]
fn integracao_busca_por_marca_samsung() {
    let c = build_real_catalog();
    let samsung = c.find_by_brand("Samsung");
    assert_eq!(samsung.len(), 3);
}

#[test]
fn integracao_busca_textual_smart_tv() {
    let c = build_real_catalog();
    let r = c.search_by_name("smart tv");
    assert_eq!(r.len(), 2);
}

#[test]
fn integracao_busca_combinada_gamer_logitech() {
    let c = build_real_catalog();
    let r = c.search(Some("gamer"), None, Some("Logitech"));
    assert_eq!(r.len(), 1);
    assert_eq!(r[0].name, "Mouse Gamer RGB");
}

#[test]
fn integracao_busca_combinada_categoria_e_texto() {
    let c = build_real_catalog();
    // Periféricos com a palavra "rgb"
    let r = c.search(Some("rgb"), Some("Periféricos"), None);
    assert_eq!(r.len(), 2); // Mouse RGB + Teclado RGB
}

#[test]
fn integracao_busca_textual_case_insensitive() {
    let c = build_real_catalog();
    let r1 = c.search_by_name("NOTEBOOK");
    let r2 = c.search_by_name("notebook");
    let r3 = c.search_by_name("Notebook");
    assert_eq!(r1.len(), r2.len());
    assert_eq!(r2.len(), r3.len());
    assert_eq!(r1.len(), 2);
}

#[test]
fn integracao_busca_sem_resultados_retorna_vazio() {
    let c = build_real_catalog();
    assert!(c.search_by_name("xyzabc inexistente").is_empty());
    assert!(c.find_by_brand("MarcaInexistente").is_empty());
    assert!(c.find_by_category("CategoriaInexistente").is_empty());
}

#[test]
fn integracao_escalabilidade_100k_produtos() {
    // Garante que adicionar 100k produtos e fazer 1000 buscas
    // por ID acontece em tempo razoável (< 1 segundo).
    use std::time::Instant;

    let mut c = Catalog::new();
    for i in 0..100_000u64 {
        c.add_product(Product::new(
            i,
            format!("Produto {}", i),
            "Marca",
            "Cat",
            1000,
            10,
        ));
    }

    let t = Instant::now();
    for i in (0..1000u64).map(|x| x * 100) {
        let _ = c.find_by_id(i);
    }
    let elapsed = t.elapsed();
    assert!(
        elapsed.as_millis() < 1000,
        "1000 buscas em 100k produtos deveriam ser < 1s, mas levaram {:?}",
        elapsed
    );
}
