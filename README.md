# Sistema de Busca Otimizado para Catálogo de Produtos — MegaStore

Sistema de busca de alta performance para o catálogo de produtos da rede de varejo MegaStore, implementado em **Rust** com uso intensivo de **tabelas hash** (`HashMap` / `HashSet`) para garantir buscas em **tempo médio O(1)**.

> Trabalho acadêmico desenvolvido para a disciplina **Data Structures Strategy and Implementation**.

---

## Descrição do projeto

A MegaStore opera um e-commerce com milhões de produtos. Seu sistema de busca tradicional retornava resultados lentos e imprecisos, frustrando os clientes e prejudicando as vendas. Este projeto entrega uma solução que:

- **Indexa** o catálogo de produtos em múltiplas dimensões (ID, nome, marca, categoria).
- Realiza **buscas rápidas e precisas** mesmo em catálogos com centenas de milhares de itens.
- É **escalável** e capaz de crescer com o catálogo da empresa.
- Cobre **filtros combinados** (texto + categoria + marca), exatamente como em um e-commerce real.

---

## Tecnologias utilizadas

| Tecnologia | Função |
|---|---|
| **Rust 2021** (edition 2021) | Linguagem de programação |
| `std::collections::HashMap` | Índice primário (ID -> Produto) e índices invertidos |
| `std::collections::HashSet` | Conjuntos de IDs por marca / categoria / palavra |
| `serde` + `serde_derive` | Serialização/deserialização (preparado para JSON) |
| `cargo` | Build system, gerenciador de pacotes e runner de testes |
| `cargo test` | Framework de testes nativo do Rust |

Nenhuma dependência externa pesada é usada — o sistema roda apenas com a biblioteca padrão do Rust.

---

## Estrutura do projeto

```
megastore_search/
├── Cargo.toml              # Configuração e dependências do projeto
├── README.md               # Este arquivo
├── src/
│   ├── lib.rs              # Biblioteca pública (reexporta os módulos)
│   ├── main.rs             # Executável de demonstração + benchmark
│   ├── product.rs          # Struct Product e métodos auxiliares
│   └── catalog.rs          # Catalog: índices hash + buscas
└── tests/
    └── integration_tests.rs # Testes de integração (sistema completo)
```

---

## Como executar o sistema

### Pré-requisitos
- **Rust 1.70+** instalado ([rustup.rs](https://rustup.rs))

### Passos

```bash
# Clonar o repositório
git clone https://github.com/reuellsfm/megastore_search.git
cd megastore_search

# Compilar
cargo build --release

# Rodar a demonstração
cargo run --release
```

A saída mostra o catálogo carregado, exemplos de cada tipo de busca e um **benchmark de escalabilidade** com 1.000, 10.000 e 100.000 produtos.

---

## Como executar os testes

```bash
# Roda todos os testes (unitários + integração + doc-tests)
cargo test

# Só os testes unitários
cargo test --lib

# Só os testes de integração
cargo test --test integration_tests

# Modo verboso (mostra prints de dentro dos testes)
cargo test -- --nocapture
```

### O que está coberto

- 20 testes unitários dentro dos módulos `product` e `catalog`
- 9 testes de integração simulando uso real
- 1 doc-test garantindo que o exemplo da documentação funciona

Total: **30 testes**, todos passando.

---

## Exemplos de uso

### Adicionando produtos ao catálogo

```rust
use megastore_search::{Catalog, Product};

let mut catalog = Catalog::new();
catalog.add_product(Product::new(
    1,
    "Notebook Gamer Pro 15",
    "Acer",
    "Eletrônicos",
    599_900, // preço em centavos
    4,
));
```

### Busca pelo ID

```rust
if let Some(produto) = catalog.find_by_id(1) {
    println!("Encontrado: {}", produto.name);
}
```

### Busca por marca ou categoria

```rust
let acer = catalog.find_by_brand("Acer");           // case-insensitive
let eletro = catalog.find_by_category("Eletrônicos");
```

### Busca textual (com múltiplas palavras — AND lógico)

```rust
let resultados = catalog.search_by_name("notebook gamer");
// retorna só os produtos que contêm AMBAS as palavras no nome
```

### Busca combinada (texto + filtros)

```rust
// "gamer" + marca "Logitech"
let resultados = catalog.search(
    Some("gamer"),
    None,                // sem filtro de categoria
    Some("Logitech")
);
```

---

## Arquitetura do sistema

O sistema é dividido em **dois módulos principais**:

### 1. `product` — modelo de dados
Define a struct `Product`, com campos como `id`, `name`, `brand`, `category`, `price_cents`, `stock`. Inclui métodos utilitários (formatação de preço, verificação de estoque).

### 2. `catalog` — motor de busca
Mantém **quatro tabelas hash** que funcionam como índices invertidos:

```
+--------------------------------------------------------------+
|  CATALOG                                                     |
|                                                              |
|   by_id        HashMap<u64,    Product>      <- índice 1:1   |
|   by_brand     HashMap<String, HashSet<u64>> <- marca -> IDs |
|   by_category  HashMap<String, HashSet<u64>> <- cat. -> IDs  |
|   name_index   HashMap<String, HashSet<u64>> <- palavra->IDs |
|                                                              |
+--------------------------------------------------------------+
```

Quando um produto é adicionado, o `Catalog` o registra **simultaneamente** em todos os índices relevantes. Isso troca um pouco de memória extra (índices duplicados) por **velocidade absurdamente maior** nas buscas.

---

## Algoritmos e estruturas de dados

### Por que tabelas hash?

| Operação | Lista linear (`Vec<Product>`) | Tabela hash (`HashMap`) |
|---|---|---|
| Busca por ID | O(n) — varre tudo | **O(1)** — acesso direto |
| Busca por marca | O(n) | **O(1)** + tamanho do resultado |
| Busca por categoria | O(n) | **O(1)** + tamanho do resultado |
| Adicionar produto | O(1) | **O(1)** amortizado |

Para 1 milhão de produtos, uma busca linear poderia precisar de **milhões** de comparações. Com tabelas hash, vamos **direto** ao item — independentemente do tamanho do catálogo.

### Busca textual: índice invertido

O `name_index` é um **índice invertido** clássico, a mesma técnica usada por motores de busca como Lucene/Elasticsearch:

```
Produto: "Notebook Gamer Pro 15"
   -> tokenização
["notebook", "gamer", "pro", "15"]
   -> inserção
name_index["notebook"] += {1}
name_index["gamer"]    += {1}
name_index["pro"]      += {1}
name_index["15"]       += {1}
```

Na consulta `"notebook gamer"`, fazemos a **interseção** dos conjuntos de IDs de cada palavra — somente os produtos que contêm **todas** as palavras passam (operador AND).

### Normalização

Todas as buscas são **case-insensitive** e ignoram pontuação. A função `tokenize` quebra o texto em palavras alfanuméricas e converte para minúsculas antes da indexação e da consulta.

---

## Considerações sobre desempenho e escalabilidade

### Resultados de benchmark (release mode, AMD64)

| Tamanho do catálogo | Busca por ID | Busca por marca | Busca textual |
|---|---|---|---|
| 1.000 produtos | 122 ns | 5 µs | 121 µs |
| 10.000 produtos | 95 ns | 89 µs | 1.380 µs |
| 100.000 produtos | 217 ns | 2.270 µs | 22.878 µs |

**Observações:**

- A **busca por ID** se mantém em nanosegundos mesmo com 100k produtos — é o comportamento O(1) das tabelas hash.
- A busca por marca/categoria cresce conforme o **tamanho do resultado** (não do catálogo), também como esperado.
- A busca textual é O(p × m), onde p é o número de palavras na consulta e m é o tamanho do menor conjunto — escala bem.

### Custo de memória

Os índices invertidos consomem **memória adicional proporcional ao número de campos indexados**. Para 100.000 produtos, o uso de RAM ficou em torno de 60 MB — perfeitamente aceitável para um servidor de e-commerce moderno.

### Limites e próximos passos

- Para catálogos até ~10 milhões de itens em uma única máquina, esta arquitetura é mais que suficiente.
- Para escalas maiores, o sistema pode ser **distribuído** (sharding por categoria) ou usar engines especializadas (Elasticsearch / Tantivy).
- Persistência (atualmente o catálogo vive em memória) pode ser adicionada com Sled ou RocksDB.

---

## Como o sistema se integra ao e-commerce

```
+---------------+    HTTP    +------------------+    busca    +----------+
|  Frontend Web | ---------> |  API (axum/etc)  | ----------> |  Catalog |
|  (React/Next) |            |  + autenticação  |             |  (Rust)  |
+---------------+ <--------- +------------------+ <---------- +----------+
                                                  results JSON
```

A biblioteca foi construída **sem amarração a um framework web** — pode ser plugada em qualquer servidor HTTP do ecossistema Rust (axum, actix-web, rocket) ou exposta via FFI para outras linguagens.

---

## Métricas de avaliação

| Métrica | Como medir | Meta |
|---|---|---|
| Tempo médio de busca | logs de latência | < 50 ms |
| Taxa de buscas com resultado | (buscas com hit) / (total) | > 90% |
| Throughput | req/s suportadas por instância | > 5.000 |
| Uso de memória | RSS do processo | < 1 GB / 1M de produtos |

---

## Contribuições

Este é um trabalho acadêmico, mas o repositório está aberto a sugestões. Para contribuir:

1. Faça um fork do projeto
2. Crie uma branch (`git checkout -b minha-feature`)
3. Faça commit das mudanças (`git commit -m 'feat: nova funcionalidade'`)
4. Abra um Pull Request

---

## Licença

Este projeto está sob a licença **MIT** — veja o arquivo `Cargo.toml` para detalhes.

---

## Autor

**Reuel S.**
Disciplina: Data Structures Strategy and Implementation · 2026
