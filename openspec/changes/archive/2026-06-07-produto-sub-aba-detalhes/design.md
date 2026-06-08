## Context

A tela de produto (`src/components/products/`) tem um container (`index.tsx`) que gerencia abas a nivel de modulo: "Lista" e uma aba dinamica para o formulario (Novo/Editar). O formulario (`ProductForm.tsx`) ja distingue modo criacao (`product === null`) de edicao.

Hoje os dados de desempenho do produto vivem em modulos separados:
- Movimentacoes de estoque: comando `list_stock_movements(product_id)` ja existe (usado por `MovementHistory.tsx`).
- Vendas: comandos `list_sales`, `get_sale_items(sale_id)` existem, mas nao ha agregacao por produto.
- Compras: `list_purchases(...)` existe, mas sem filtro por produto.
- Precificacao: o breakdown e calculado no cliente por `src/lib/pricing.ts` (`computeMarginFromPrice`) a partir dos campos do `Product` e das `PricingRates`. O `ProductForm` ja renderiza esse breakdown.

A mudanca consolida vendas, estoque, compras e precificacao numa sub-aba "Detalhes" dentro da tela de edicao de produto.

## Goals / Non-Goals

**Goals:**
- Sub-abas internas Geral/Detalhes no modo edicao, sem trocar de modulo.
- Reaproveitar `list_stock_movements` e `src/lib/pricing.ts` sem duplicar logica.
- Adicionar dois comandos de leitura por produto: vendas e compras.
- Manter o formulario atual (Geral) intacto em comportamento.

**Non-Goals:**
- A sub-aba "Detalhes" e estritamente de leitura: nao tera campos editaveis nem acoes de escrita; nenhuma edicao de produto migra do form Geral para Detalhes.
- Nenhuma mudanca de schema DuckDB (apenas SELECTs sobre tabelas existentes).
- Nao adicionar graficos/series temporais elaboradas (evolucao de preco fica como listagem simples, nao grafico).
- Nao mexer no modulo Estoque > Movimentacao nem na tela de Vendas.
- Sub-aba Detalhes nao aparece no modo "Novo produto".

## Decisions

### Decisao 1: Sub-abas dentro do ProductForm vs. novo wrapper

Escolha: extrair um componente `ProductDetailTabs` que envolve o `ProductForm` (Geral) e o novo `ProductDetails` (Detalhes), renderizado pelo `index.tsx` apenas quando `formContent.kind === "edit"`. No modo "new", `index.tsx` renderiza `ProductForm` direto, sem sub-abas.

Alternativa considerada: colocar a logica de sub-abas dentro do proprio `ProductForm`. Rejeitada: aumentaria a complexidade de um componente ja grande (540+ linhas) e misturaria responsabilidades (form de edicao vs. visualizacao read-only).

Padrao de tab segue o ja usado em `index.tsx` (`tabClass`), para consistencia visual.

### Decisao 2: Backend - novos comandos por produto

`get_product_sales_history(product_id) -> Vec<ProductSaleRow>`: SELECT em `Sale_Items` join `Sales` filtrando por `product_id`, retornando por venda (ou agregado): data, quantidade, preco unitario, receita, status da venda. Apenas vendas fechadas (`status = closed`) contam para receita realizada; rascunhos podem ser exibidos separadamente ou omitidos (decisao: omitir rascunhos no historico).

`list_product_purchases(product_id) -> Vec<ProductPurchaseRow>`: SELECT em itens de compra join `Purchases` filtrando por `product_id`, retornando data, fornecedor, quantidade, preco unitario, total. So compras fechadas.

Alternativa: estender `list_purchases` com parametro opcional `product_id`. Rejeitada: a assinatura atual ja serve a outra tela; um comando dedicado de leitura por produto e mais simples de testar e nao arrisca regressao.

Cada novo comando recebe ao menos um teste unitario Rust `#[cfg(test)]` no mesmo arquivo (regra do projeto).

### Decisao 3: Precificacao reusa pricing.ts

A secao de precificacao em Detalhes reusa `computeMarginFromPrice`/`computeCmvFromComponents` de `src/lib/pricing.ts`, exibindo CMV, CV, DF, investimento, margem, markup e preco final. Como o `ProductForm` ja faz esse calculo, extrair uma funcao pura de "breakdown completo" para `src/lib/pricing.ts` (se ainda nao existir uma que retorne todos os campos) e cobri-la com teste Vitest. Sem duplicar formula no componente.

### Decisao 4: Carregamento de dados

`ProductDetails` carrega as quatro secoes via `invoke` em paralelo (`Promise.all`) ao montar, com estado de loading por secao (ou um spinner global). Sem refetch ao alternar entre Geral/Detalhes (ambos montados, alternados por `hidden`, seguindo o padrao de `index.tsx`).

## Risks / Trade-offs

- [Volume de dados: produtos com muito historico podem retornar listas longas] -> limitar/ordenar por data desc no SQL (ex.: ultimas N ou ordenado decrescente) e paginar no cliente se necessario.
- [Insumo nao-vendavel nao tem vendas; produto vendavel nao tem compras diretas] -> cada secao trata vazio com mensagem propria; secoes irrelevantes ao tipo do produto podem ser ocultadas (ex.: "Historico de vendas" so para `sellable`, "Historico de compras" so para insumos).
- [Divergencia de calculo de precificacao entre Geral e Detalhes] -> mitigado extraindo uma unica funcao pura em `pricing.ts` usada pelos dois.

## Migration Plan

Sem migracao de dados. Mudanca aditiva: novos comandos e UI. Rollback = reverter os arquivos novos e o registro dos comandos no `invoke_handler`.

## Open Questions

- Historico de vendas deve ser por linha de venda ou agregado por periodo (dia/mes)? Decisao inicial: por venda, ordenado por data desc.
- Exibir rascunhos de venda/compra? Decisao inicial: nao, apenas fechados.
