//! Módulo `product`
//!
//! Define a estrutura `Product` (Produto), que representa cada item do
//! catálogo da MegaStore. A struct é projetada para ser leve, clonável
//! e serializável, facilitando a busca em diferentes índices.

use serde::{Deserialize, Serialize};

/// Representa um produto no catálogo da MegaStore.
///
/// Cada produto contém um identificador único (`id`), além de campos
/// indexáveis como `name`, `brand` e `category`.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Product {
    /// Identificador único do produto.
    pub id: u64,
    /// Nome do produto (ex: "Notebook Gamer Pro 15").
    pub name: String,
    /// Marca/fabricante do produto.
    pub brand: String,
    /// Categoria (ex: "Eletrônicos", "Vestuário").
    pub category: String,
    /// Preço em reais (centavos para evitar erros de ponto flutuante).
    pub price_cents: u64,
    /// Estoque disponível.
    pub stock: u32,
}

impl Product {
    /// Cria um novo produto.
    pub fn new(
        id: u64,
        name: impl Into<String>,
        brand: impl Into<String>,
        category: impl Into<String>,
        price_cents: u64,
        stock: u32,
    ) -> Self {
        Self {
            id,
            name: name.into(),
            brand: brand.into(),
            category: category.into(),
            price_cents,
            stock,
        }
    }

    /// Retorna o preço formatado em reais (R$ X,XX).
    pub fn price_formatted(&self) -> String {
        let reais = self.price_cents / 100;
        let cents = self.price_cents % 100;
        format!("R$ {},{:02}", reais, cents)
    }

    /// Indica se o produto está em estoque.
    pub fn in_stock(&self) -> bool {
        self.stock > 0
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn cria_produto_corretamente() {
        let p = Product::new(1, "Mouse", "Logitech", "Periféricos", 14999, 10);
        assert_eq!(p.id, 1);
        assert_eq!(p.name, "Mouse");
        assert_eq!(p.brand, "Logitech");
        assert_eq!(p.category, "Periféricos");
        assert_eq!(p.price_cents, 14999);
        assert_eq!(p.stock, 10);
    }

    #[test]
    fn formata_preco_corretamente() {
        let p = Product::new(1, "X", "Y", "Z", 14999, 1);
        assert_eq!(p.price_formatted(), "R$ 149,99");
    }

    #[test]
    fn preco_com_zero_centavos() {
        let p = Product::new(1, "X", "Y", "Z", 10000, 1);
        assert_eq!(p.price_formatted(), "R$ 100,00");
    }

    #[test]
    fn detecta_estoque_vazio() {
        let p = Product::new(1, "X", "Y", "Z", 100, 0);
        assert!(!p.in_stock());
    }

    #[test]
    fn detecta_estoque_disponivel() {
        let p = Product::new(1, "X", "Y", "Z", 100, 5);
        assert!(p.in_stock());
    }
}
