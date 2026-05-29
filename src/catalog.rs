//! Módulo `catalog`
//!
//! Implementa o `Catalog` (Catálogo), núcleo do sistema de busca.
//!
//! ## Arquitetura
//!
//! Para garantir buscas em tempo médio O(1) — independente do tamanho
//! do catálogo —, o `Catalog` mantém múltiplas tabelas hash atuando
//! como **índices invertidos**:
//!
//! - `by_id`         → busca direta pelo ID do produto
//! - `by_brand`      → todos os produtos de uma marca
//! - `by_category`   → todos os produtos de uma categoria
//! - `name_index`    → tokens (palavras) do nome → IDs dos produtos
//!
//! Essa estrutura é o que diferencia uma busca otimizada de uma busca
//! linear: em vez de varrer o catálogo inteiro a cada consulta, vamos
//! direto ao "balde" certo via hashing.

use crate::product::Product;
use std::collections::{HashMap, HashSet};

/// Catálogo de produtos com índices internos para buscas rápidas.
#[derive(Debug, Default)]
pub struct Catalog {
    /// Índice principal: ID → Produto.
    by_id: HashMap<u64, Product>,
    /// Índice por marca (em minúsculas) → conjunto de IDs.
    by_brand: HashMap<String, HashSet<u64>>,
    /// Índice por categoria (em minúsculas) → conjunto de IDs.
    by_category: HashMap<String, HashSet<u64>>,
    /// Índice invertido por palavra do nome → conjunto de IDs.
    name_index: HashMap<String, HashSet<u64>>,
}

impl Catalog {
    /// Cria um novo catálogo vazio.
    pub fn new() -> Self {
        Self::default()
    }

    /// Total de produtos no catálogo.
    pub fn len(&self) -> usize {
        self.by_id.len()
    }

    /// `true` se o catálogo está vazio.
    pub fn is_empty(&self) -> bool {
        self.by_id.is_empty()
    }

    /// Adiciona um produto ao catálogo, atualizando todos os índices.
    ///
    /// Complexidade: O(k), onde k é o número de palavras no nome
    /// (independente do tamanho do catálogo).
    pub fn add_product(&mut self, product: Product) {
        let id = product.id;

        // Índices secundários — usamos minúsculas para busca case-insensitive
        self.by_brand
            .entry(normalize(&product.brand))
            .or_default()
            .insert(id);

        self.by_category
            .entry(normalize(&product.category))
            .or_default()
            .insert(id);

        for token in tokenize(&product.name) {
            self.name_index.entry(token).or_default().insert(id);
        }

        // Por último, insere no índice primário (assume posse do produto)
        self.by_id.insert(id, product);
    }

    /// Adiciona vários produtos de uma vez (mais ergonômico).
    pub fn add_products<I: IntoIterator<Item = Product>>(&mut self, items: I) {
        for p in items {
            self.add_product(p);
        }
    }

    /// Remove um produto do catálogo (mantendo os índices consistentes).
    pub fn remove_product(&mut self, id: u64) -> Option<Product> {
        let product = self.by_id.remove(&id)?;

        if let Some(set) = self.by_brand.get_mut(&normalize(&product.brand)) {
            set.remove(&id);
        }
        if let Some(set) = self.by_category.get_mut(&normalize(&product.category)) {
            set.remove(&id);
        }
        for token in tokenize(&product.name) {
            if let Some(set) = self.name_index.get_mut(&token) {
                set.remove(&id);
            }
        }

        Some(product)
    }

    // ----------------- BUSCAS -----------------

    /// Busca produto pelo ID. Tempo médio: O(1).
    pub fn find_by_id(&self, id: u64) -> Option<&Product> {
        self.by_id.get(&id)
    }

    /// Lista todos os produtos de uma determinada marca.
    /// Tempo médio: O(1) para localizar + O(n) para coletar n resultados.
    pub fn find_by_brand(&self, brand: &str) -> Vec<&Product> {
        self.by_brand
            .get(&normalize(brand))
            .map(|ids| ids.iter().filter_map(|i| self.by_id.get(i)).collect())
            .unwrap_or_default()
    }

    /// Lista todos os produtos de uma categoria.
    pub fn find_by_category(&self, category: &str) -> Vec<&Product> {
        self.by_category
            .get(&normalize(category))
            .map(|ids| ids.iter().filter_map(|i| self.by_id.get(i)).collect())
            .unwrap_or_default()
    }

    /// Busca textual no nome do produto.
    ///
    /// Suporta múltiplas palavras: retorna produtos que contenham
    /// **TODAS** as palavras informadas (AND lógico).
    /// Tempo médio: O(p × m), onde p é o número de palavras da query
    /// e m é o tamanho do menor conjunto de resultados.
    pub fn search_by_name(&self, query: &str) -> Vec<&Product> {
        let tokens: Vec<String> = tokenize(query);
        if tokens.is_empty() {
            return Vec::new();
        }

        // Pega o conjunto de IDs do primeiro token
        let mut result_ids: HashSet<u64> = match self.name_index.get(&tokens[0]) {
            Some(set) => set.clone(),
            None => return Vec::new(),
        };

        // Faz interseção com os conjuntos dos demais tokens (AND)
        for token in tokens.iter().skip(1) {
            match self.name_index.get(token) {
                Some(set) => {
                    result_ids = result_ids.intersection(set).copied().collect();
                }
                None => return Vec::new(),
            }
            if result_ids.is_empty() {
                return Vec::new();
            }
        }

        result_ids
            .iter()
            .filter_map(|id| self.by_id.get(id))
            .collect()
    }

    /// Busca combinada: nome + categoria + marca (qualquer combinação).
    ///
    /// Aplica filtros sucessivos. Útil para o front-end com filtros
    /// laterais (categoria/marca) e barra de busca por texto.
    pub fn search(
        &self,
        name_query: Option<&str>,
        category: Option<&str>,
        brand: Option<&str>,
    ) -> Vec<&Product> {
        // Começamos do índice mais restritivo, se houver
        let mut ids: Option<HashSet<u64>> = None;

        if let Some(q) = name_query.filter(|s| !s.trim().is_empty()) {
            let r: HashSet<u64> = self
                .search_by_name(q)
                .iter()
                .map(|p| p.id)
                .collect();
            ids = Some(merge(ids, r));
        }
        if let Some(c) = category {
            let r: HashSet<u64> = self
                .by_category
                .get(&normalize(c))
                .cloned()
                .unwrap_or_default();
            ids = Some(merge(ids, r));
        }
        if let Some(b) = brand {
            let r: HashSet<u64> = self
                .by_brand
                .get(&normalize(b))
                .cloned()
                .unwrap_or_default();
            ids = Some(merge(ids, r));
        }

        match ids {
            // Nenhum filtro foi aplicado: retorna todos
            None => self.by_id.values().collect(),
            Some(set) => set
                .iter()
                .filter_map(|id| self.by_id.get(id))
                .collect(),
        }
    }
}

// ----------------- Funções auxiliares -----------------

/// Normaliza uma string: minúsculas + sem espaços nas pontas.
fn normalize(s: &str) -> String {
    s.trim().to_lowercase()
}

/// Quebra uma string em tokens (palavras), em minúsculas, sem pontuação.
fn tokenize(s: &str) -> Vec<String> {
    s.split(|c: char| !c.is_alphanumeric())
        .filter(|w| !w.is_empty())
        .map(|w| w.to_lowercase())
        .collect()
}

/// Intersecta o `acc` (acumulador) com um novo conjunto.
fn merge(acc: Option<HashSet<u64>>, new: HashSet<u64>) -> HashSet<u64> {
    match acc {
        None => new,
        Some(prev) => prev.intersection(&new).copied().collect(),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn sample_catalog() -> Catalog {
        let mut c = Catalog::new();
        c.add_products(vec![
            Product::new(1, "Notebook Gamer Pro 15", "Acer", "Eletrônicos", 599900, 4),
            Product::new(2, "Notebook Office Basic", "Acer", "Eletrônicos", 299900, 8),
            Product::new(3, "Mouse Gamer RGB", "Logitech", "Periféricos", 24900, 50),
            Product::new(4, "Teclado Mecânico", "Logitech", "Periféricos", 35000, 20),
            Product::new(5, "Camiseta Básica", "Hering", "Vestuário", 4990, 100),
        ]);
        c
    }

    #[test]
    fn catalogo_vazio_inicialmente() {
        let c = Catalog::new();
        assert!(c.is_empty());
        assert_eq!(c.len(), 0);
    }

    #[test]
    fn adiciona_e_conta_produtos() {
        let c = sample_catalog();
        assert_eq!(c.len(), 5);
    }

    #[test]
    fn busca_por_id() {
        let c = sample_catalog();
        let p = c.find_by_id(3).expect("produto 3 deve existir");
        assert_eq!(p.name, "Mouse Gamer RGB");
    }

    #[test]
    fn busca_por_id_inexistente_retorna_none() {
        let c = sample_catalog();
        assert!(c.find_by_id(999).is_none());
    }

    #[test]
    fn busca_por_marca_traz_todos() {
        let c = sample_catalog();
        let acer = c.find_by_brand("Acer");
        assert_eq!(acer.len(), 2);
        let logitech = c.find_by_brand("Logitech");
        assert_eq!(logitech.len(), 2);
    }

    #[test]
    fn busca_por_marca_case_insensitive() {
        let c = sample_catalog();
        assert_eq!(c.find_by_brand("ACER").len(), 2);
        assert_eq!(c.find_by_brand("acer").len(), 2);
    }

    #[test]
    fn busca_por_categoria() {
        let c = sample_catalog();
        let perifericos = c.find_by_category("Periféricos");
        assert_eq!(perifericos.len(), 2);
    }

    #[test]
    fn busca_textual_uma_palavra() {
        let c = sample_catalog();
        let r = c.search_by_name("notebook");
        assert_eq!(r.len(), 2); // Notebook Gamer + Notebook Office
    }

    #[test]
    fn busca_textual_multiplas_palavras_faz_and() {
        let c = sample_catalog();
        let r = c.search_by_name("notebook gamer");
        assert_eq!(r.len(), 1); // só o "Notebook Gamer Pro 15"
        assert_eq!(r[0].id, 1);
    }

    #[test]
    fn busca_textual_sem_resultados() {
        let c = sample_catalog();
        let r = c.search_by_name("inexistente");
        assert!(r.is_empty());
    }

    #[test]
    fn busca_combinada_marca_e_categoria() {
        let c = sample_catalog();
        let r = c.search(None, Some("Periféricos"), Some("Logitech"));
        assert_eq!(r.len(), 2);
    }

    #[test]
    fn busca_combinada_texto_e_marca() {
        let c = sample_catalog();
        let r = c.search(Some("gamer"), None, Some("Logitech"));
        assert_eq!(r.len(), 1);
        assert_eq!(r[0].name, "Mouse Gamer RGB");
    }

    #[test]
    fn busca_sem_filtros_retorna_todos() {
        let c = sample_catalog();
        let r = c.search(None, None, None);
        assert_eq!(r.len(), 5);
    }

    #[test]
    fn remover_produto_atualiza_indices() {
        let mut c = sample_catalog();
        let removido = c.remove_product(1);
        assert!(removido.is_some());
        assert_eq!(c.len(), 4);

        // O "Notebook Gamer" foi removido — busca não deve achar mais
        let r = c.search_by_name("notebook gamer");
        assert!(r.is_empty());

        // Mas "notebook" sozinho ainda acha o Office
        let r = c.search_by_name("notebook");
        assert_eq!(r.len(), 1);
        assert_eq!(r[0].id, 2);
    }

    #[test]
    fn tokenize_ignora_pontuacao_e_espacos() {
        let r = tokenize("Notebook, Gamer-Pro 15!");
        assert_eq!(r, vec!["notebook", "gamer", "pro", "15"]);
    }
}
