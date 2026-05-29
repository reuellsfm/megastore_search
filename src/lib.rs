//! # MegaStore Search
//!
//! Biblioteca de busca otimizada para o catálogo de produtos da MegaStore.
//!
//! O sistema utiliza **tabelas hash** (`HashMap` / `HashSet` da stdlib do Rust)
//! para indexar produtos por múltiplos critérios e oferecer buscas em
//! **tempo médio O(1)**.
//!
//! ## Exemplo rápido
//!
//! ```
//! use megastore_search::{Catalog, Product};
//!
//! let mut catalog = Catalog::new();
//! catalog.add_product(Product::new(1, "Notebook Gamer", "Acer", "Eletrônicos", 599900, 4));
//!
//! let results = catalog.search_by_name("notebook");
//! assert_eq!(results.len(), 1);
//! ```

pub mod catalog;
pub mod product;

// Reexporta os tipos principais para uso conveniente
pub use catalog::Catalog;
pub use product::Product;
