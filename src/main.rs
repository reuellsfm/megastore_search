//! Executável de demonstração do sistema de busca da MegaStore.
//!
//! Cria um catálogo de exemplo com dezenas de produtos e demonstra
//! cada tipo de busca, exibindo os tempos de execução.

use megastore_search::{Catalog, Product};
use std::time::Instant;

fn main() {
    println!("┌────────────────────────────────────────────────────┐");
    println!("│   MegaStore — Sistema de Busca Otimizado          │");
    println!("│   Estruturas de Dados: HashMap + HashSet (Rust)   │");
    println!("└────────────────────────────────────────────────────┘\n");

    // 1) Monta o catálogo
    let mut catalog = Catalog::new();
    populate(&mut catalog);
    println!("✓ Catálogo carregado com {} produtos\n", catalog.len());

    // 2) Busca por ID
    demo_section("Busca por ID");
    let t = Instant::now();
    if let Some(p) = catalog.find_by_id(3) {
        println!(
            "  ID 3 → {} [{}] · {}",
            p.name,
            p.category,
            p.price_formatted()
        );
    }
    print_elapsed(t);

    // 3) Busca por marca
    demo_section("Busca por marca: 'Acer'");
    let t = Instant::now();
    let acer = catalog.find_by_brand("Acer");
    for p in &acer {
        println!("  • {} · {} · estoque: {}", p.name, p.price_formatted(), p.stock);
    }
    print_elapsed(t);

    // 4) Busca por categoria
    demo_section("Busca por categoria: 'Periféricos'");
    let t = Instant::now();
    let perif = catalog.find_by_category("Periféricos");
    for p in &perif {
        println!("  • {} ({}) · {}", p.name, p.brand, p.price_formatted());
    }
    print_elapsed(t);

    // 5) Busca textual
    demo_section("Busca textual: 'notebook gamer'");
    let t = Instant::now();
    let results = catalog.search_by_name("notebook gamer");
    for p in &results {
        println!("  • {} ({}) · {}", p.name, p.brand, p.price_formatted());
    }
    print_elapsed(t);

    // 6) Busca combinada (texto + filtros)
    demo_section("Busca combinada: 'gamer' + marca 'Logitech'");
    let t = Instant::now();
    let results = catalog.search(Some("gamer"), None, Some("Logitech"));
    for p in &results {
        println!("  • {} · {}", p.name, p.price_formatted());
    }
    print_elapsed(t);

    // 7) Benchmark: cresce o catálogo e mede o tempo de busca
    demo_section("Benchmark de escalabilidade");
    benchmark();
}

fn populate(c: &mut Catalog) {
    c.add_products(vec![
        Product::new(1, "Notebook Gamer Pro 15", "Acer", "Eletrônicos", 599900, 4),
        Product::new(2, "Notebook Office Basic 14", "Acer", "Eletrônicos", 299900, 8),
        Product::new(3, "Mouse Gamer RGB Pro", "Logitech", "Periféricos", 24900, 50),
        Product::new(4, "Teclado Mecânico", "Logitech", "Periféricos", 35000, 20),
        Product::new(5, "Headset Gamer Surround", "HyperX", "Periféricos", 45000, 15),
        Product::new(6, "Camiseta Básica Algodão", "Hering", "Vestuário", 4990, 100),
        Product::new(7, "Calça Jeans Slim", "Levi's", "Vestuário", 19990, 40),
        Product::new(8, "Tênis Esportivo", "Nike", "Calçados", 29990, 25),
        Product::new(9, "Smart TV 55 4K", "Samsung", "Eletrônicos", 289900, 6),
        Product::new(10, "Cafeteira Elétrica", "Britânia", "Eletrodomésticos", 12990, 30),
    ]);
}

fn benchmark() {
    let sizes = [1_000, 10_000, 100_000];

    for &size in &sizes {
        let mut c = Catalog::new();
        for i in 0..size as u64 {
            let cat = ["Eletrônicos", "Vestuário", "Periféricos", "Calçados"][(i % 4) as usize];
            let brand = ["Acer", "Logitech", "Nike", "Samsung", "Sony"][(i % 5) as usize];
            c.add_product(Product::new(
                i,
                format!("Produto Teste {}", i),
                brand,
                cat,
                10000 + i,
                10,
            ));
        }
        let t = Instant::now();
        let r = c.find_by_id(size as u64 / 2);
        let id_us = t.elapsed().as_nanos();

        let t = Instant::now();
        let _r2 = c.find_by_brand("Acer");
        let brand_us = t.elapsed().as_micros();

        let t = Instant::now();
        let _r3 = c.search_by_name("produto teste");
        let text_us = t.elapsed().as_micros();

        println!(
            "  {} produtos → ID: {} ns · Marca: {} µs · Texto: {} µs · achou: {}",
            size,
            id_us,
            brand_us,
            text_us,
            r.is_some()
        );
    }
}

fn demo_section(title: &str) {
    println!("\n─── {} ───", title);
}

fn print_elapsed(t: Instant) {
    let micros = t.elapsed().as_micros();
    println!("  └─ tempo: {} µs", micros);
}
