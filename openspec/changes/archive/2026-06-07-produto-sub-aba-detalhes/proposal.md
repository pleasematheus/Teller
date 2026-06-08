## Why

Ao editar um produto, o usuario so ve o formulario de cadastro/precificacao. Para entender o desempenho de um produto (quanto vendeu, como o estoque se moveu, historico de compras do insumo e o detalhamento completo da precificacao) e preciso navegar por varios modulos diferentes (Vendas, Estoque, Compras), perdendo o contexto do produto. Consolidar essas informacoes numa sub-aba "Detalhes" dentro da propria tela do produto da uma visao 360 graus sem trocar de contexto.

## What Changes

- Ao abrir um produto existente (modo edicao), a area do formulario passa a ter duas sub-abas internas: **Geral** (o formulario atual, sem alteracoes) e **Detalhes** (nova).
- A sub-aba **Detalhes** so existe no modo edicao (produto ja persistido). No modo "Novo produto" nenhuma sub-aba aparece, apenas o formulario atual.
- A sub-aba **Detalhes** reune quatro secoes para o produto selecionado:
  - **Historico de vendas**: quantidade vendida, receita e datas das vendas que incluem o produto.
  - **Movimentacoes de estoque**: entradas, saidas e ajustes (reusa a logica de `list_stock_movements`).
  - **Historico de compras**: compras do insumo com fornecedor, preco unitario, data e evolucao do custo.
  - **Detalhamento de precificacao**: breakdown completo (CMV, CV, DF, investimento, margem, markup, preco final).
- Novos comandos Tauri para agregar dados por produto:
  - `get_product_sales_history(product_id)`: itens de venda agregados/listados por produto.
  - `list_product_purchases(product_id)`: compras filtradas por produto/insumo.
- A aba ao nivel do modulo "Movimentacao" no Estoque permanece inalterada; esta mudanca apenas reaproveita a consulta por produto.

## Capabilities

### New Capabilities
- `produto-detalhes`: sub-aba de informacoes detalhadas (vendas, estoque, compras, precificacao) dentro da tela de edicao de produto, incluindo os comandos de backend que a alimentam.

### Modified Capabilities
- `product-management`: a tela de produto ganha navegacao por sub-abas (Geral/Detalhes) no modo edicao; o requisito de UI da tela muda.

## Impact

- Frontend:
  - `src/components/products/index.tsx`: ja gerencia abas a nivel de modulo; nenhuma alteracao estrutural obrigatoria.
  - `src/components/products/ProductForm.tsx`: passa a renderizar sub-abas internas (Geral/Detalhes) quando em modo edicao, ou extrai-se um wrapper `ProductDetailTabs`.
  - Novo componente `src/components/products/ProductDetails.tsx` (e sub-componentes por secao).
  - Reuso de `src/lib/pricing.ts` para o breakdown de precificacao.
- Backend:
  - `src-tauri/src/sales.rs`: novo comando `get_product_sales_history`.
  - `src-tauri/src/stock.rs`: novo comando `list_product_purchases` (ou novo parametro de filtro em `list_purchases`).
  - Registro dos novos comandos no `invoke_handler`.
- Tipos: novos tipos em `src/types/` para as linhas de historico de vendas e compras por produto.
- Sem mudanca de schema DuckDB (consultas sobre tabelas existentes de vendas, itens de venda, movimentos e compras).
