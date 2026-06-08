## 1. Backend - comandos por produto

- [x] 1.1 Ler `src-tauri/src/sales.rs` (estrutura de `Sale_Items`/`Sales`, tipos de retorno existentes) antes de editar
- [x] 1.2 Definir tipo `ProductSaleRow` (data, quantidade, preco unitario, receita) com `#[derive(Serialize)]`
- [x] 1.3 Implementar comando `get_product_sales_history(product_id)` em `sales.rs`: SELECT em `Sale_Items` join `Sales` por `product_id`, apenas vendas fechadas, ordenado por data desc
- [x] 1.4 Teste unitario Rust `#[cfg(test)]` em `sales.rs` para `get_product_sales_history` (produto com vendas, produto sem vendas, rascunho ignorado)
- [x] 1.5 Ler `src-tauri/src/stock.rs` (estrutura de itens de compra/`Purchases`, `list_purchases`) antes de editar
- [x] 1.6 Definir tipo `ProductPurchaseRow` (data, fornecedor, quantidade, preco unitario, total)
- [x] 1.7 Implementar comando `list_product_purchases(product_id)` em `stock.rs`: SELECT itens de compra join `Purchases` por `product_id`, apenas compras fechadas, ordenado por data desc
- [x] 1.8 Teste unitario Rust `#[cfg(test)]` em `stock.rs` para `list_product_purchases` (insumo com compras, produto sem compras)
- [x] 1.9 Registrar `get_product_sales_history` e `list_product_purchases` no `invoke_handler` (lib.rs/main.rs)

## 2. Tipos e logica de precificacao (frontend)

- [x] 2.1 Adicionar tipos `ProductSaleRow` e `ProductPurchaseRow` em `src/types/` (alinhados aos structs Rust)
- [x] 2.2 Ler `src/lib/pricing.ts` e verificar se ha funcao pura que retorne breakdown completo (CMV, CV, DF, investimento, margem, markup, preco final)
- [x] 2.3 Se necessario, extrair/expor funcao pura de breakdown completo em `src/lib/pricing.ts` reutilizando `computeMarginFromPrice`/`computeCmvFromComponents`
- [x] 2.4 Teste Vitest para a funcao de breakdown completo em `src/lib/pricing.test.ts` (casos: sem componentes, com componentes, preco abaixo do custo)

## 3. UI - sub-abas e secao Detalhes

- [x] 3.1 Criar `ProductDetailTabs` (wrapper) que renderiza sub-abas "Geral"/"Detalhes" usando o padrao `tabClass` existente, com "Geral" ativa por padrao
- [x] 3.2 Em `src/components/products/index.tsx`, renderizar `ProductDetailTabs` quando `formContent.kind === "edit"` e `ProductForm` direto quando `kind === "new"`
- [x] 3.3 Garantir que alternar entre "Geral" e "Detalhes" preserve o estado do `ProductForm` (montar ambos, alternar por `hidden`)
- [x] 3.4 Criar `src/components/products/ProductDetails.tsx` que recebe o `product` e carrega as quatro secoes em paralelo (`Promise.all`) com estado de loading
- [x] 3.5 Secao "Historico de vendas": consumir `get_product_sales_history`, exibir tabela (data, qtd, preco unit., receita) e mensagem de vazio; ocultar/condicionar para produto vendavel
- [x] 3.6 Secao "Movimentacoes de estoque": consumir `list_stock_movements(product_id)` automaticamente para o produto em contexto, reutilizando a apresentacao de `MovementHistory` sem o seletor de produto
- [x] 3.7 Secao "Historico de compras": consumir `list_product_purchases`, exibir tabela (data, fornecedor, qtd, preco unit., total) e mensagem de vazio; condicionar para insumo
- [x] 3.8 Secao "Detalhamento de precificacao": usar a funcao pura de breakdown e exibir CMV, CV, DF, investimento, margem, markup, preco final

## 4. Validacao

- [x] 4.1 Rodar `cargo test` e confirmar testes dos novos comandos passando (delegar a subagente, reportar resultado)
- [x] 4.2 Rodar `bun run test` e confirmar suite frontend (incluindo novos testes de pricing) passando
- [x] 4.3 Type-check do frontend sem novos erros introduzidos
- [x] 4.4 [MANUAL - testador humano] Editar um produto vendavel: confirmar sub-abas Geral/Detalhes, dados de vendas e precificacao corretos
- [x] 4.5 [MANUAL - testador humano] Editar um insumo: confirmar historico de compras e movimentacoes de estoque corretos
- [x] 4.6 [MANUAL - testador humano] Criar novo produto: confirmar ausencia de sub-abas e formulario inalterado
