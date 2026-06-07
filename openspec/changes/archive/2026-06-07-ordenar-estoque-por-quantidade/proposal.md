## Why

A tela de estoque (StockBalance) lista produtos com saldo, mas sem ordenação. Para identificar rapidamente itens com baixo/alto estoque, maior capacidade produtiva, ou agrupar por unidade/nome, o usuário precisa varrer a tabela manualmente. Tornar todas as colunas ordenáveis resolve isso.

## What Changes

- Tornar todas as colunas de dados da tabela de estoque ordenáveis: Produto (nome), Unidade, Quantidade e Capacidade estimada.
- Cada cabeçalho de coluna vira clicável: alterna entre ascendente, descendente e sem ordenação (volta à ordem padrão). Apenas uma coluna fica ativa por vez; clicar em outra coluna inicia em ascendente.
- Indicador visual (seta) no cabeçalho ativo mostra a direção.
- Critérios por coluna:
  - Produto e Unidade: ordenação textual (localeCompare pt-BR, sensível a acento na colação).
  - Quantidade: numérica por `current_quantity`.
  - Capacidade estimada: numérica pela MAIOR capacidade da linha; linhas sem capacidade ("—") vão sempre para o fim.
- A ordenação opera sobre o conjunto já filtrado (tipo) e buscado (nome). Quando há ordenação ativa, ela tem precedência sobre o ranking por relevância de nome da busca.

## Capabilities

### New Capabilities

(nenhuma)

### Modified Capabilities

- `stock-management`: adiciona requisito de ordenação por coluna na listagem de saldo de estoque.

## Impact

- `src/components/stock/StockBalance.tsx`: estado de ordenação (coluna + direção), cabeçalhos clicáveis, config de colunas com acessor de valor, composição filtro -> busca -> ordenação.
- `src/lib/listSearch.ts`: função pura genérica de ordenação por acessor (string/number) com nulos ao fim.
- Testes: casos novos em `src/lib/listSearch.test.ts`.
- Sem mudança de backend, schema DuckDB ou comandos Tauri.
