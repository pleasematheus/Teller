## Context

`Stock_Movements` registra toda entrada/saída de estoque com `product_id`, `quantity`, `movement_type`, `motive` e `created_at`. Quando uma venda é fechada (`close_sale` em `sales.rs`), o sistema cria movimentações `out`/`venda`: uma por produto sem componentes, ou uma por insumo consumido via `Product_Components` (BOM). Nenhuma dessas movimentações guarda o `sale_id`, então a tela de histórico (`MovementHistory`) não consegue ligar uma baixa de insumo à venda que a gerou.

O banco é DuckDB embutido (bundled). Conforme as diretrizes do projeto, mudanças de schema exigem tarefa de apagar a versão antiga do banco e recriar.

## Goals / Non-Goals

**Goals:**
- Rastrear cada baixa de venda até a venda de origem via `sale_id` em `Stock_Movements`.
- Exibir o ID da venda no histórico de movimentações da tela de estoque.

**Non-Goals:**
- Ligar movimentações de compra a `purchase_id` (fora de escopo; pode ser feito depois seguindo o mesmo padrão).
- Mudar o cálculo de saldo (view `Stock`) ou a lógica de consumo de BOM.
- Migração de dados históricos: registros antigos permanecem com `sale_id = NULL`.

## Decisions

**Coluna `sale_id` nullable com FK para `Sales(id)`.**
Nullable porque compra, perda e ajuste não têm venda associada. FK garante integridade referencial. Alternativa rejeitada: tabela de ligação separada — excesso de complexidade para um campo 1:1 opcional.

**Preencher `sale_id` no `close_sale`, nos dois caminhos de baixa.**
Os dois `INSERT INTO "Stock_Movements"` em `sales.rs` (produto sem componentes e insumo via BOM) já estão dentro da transação de fechamento e têm `sale_id` em escopo. Basta acrescentar a coluna e o parâmetro.

**Exibir `sale_id` como coluna nova na tabela de `MovementHistory`.**
Mostrar `#<id>` quando presente, vazio caso contrário. Sem link de navegação por ora (a tela de vendas fechadas é separada).

**Recriar a tabela no schema, sem `ALTER TABLE`.**
O schema é versionado e o banco é recriado em dev. Adiciona-se a coluna na definição de `CREATE TABLE "Stock_Movements"`. Tarefa explícita de apagar o `.db` antigo e recriar.

## Risks / Trade-offs

- [Inserts de teste existentes em `sales.rs`/`stock.rs` omitem `sale_id`] → coluna é NULL-able, então inserts que não listam a coluna seguem válidos; nenhum teste quebra.
- [Registros históricos ficam sem `sale_id`] → aceitável; auditoria passa a valer da migração em diante. UI trata NULL como vazio.
- [FK para `Sales` pode falhar se a venda for apagada] → o sistema não apaga vendas fechadas; risco baixo. Sem `ON DELETE` especial, mantendo padrão das outras FKs do schema.

## Migration Plan

1. Atualizar `CREATE TABLE "Stock_Movements"` em `schema.sql` com `sale_id INTEGER` + FK.
2. Apagar o arquivo de banco DuckDB local de desenvolvimento.
3. Recriar o banco rodando o app (schema aplicado na inicialização).
4. Rollback: reverter o schema e recriar o banco.

## Open Questions

(nenhuma)
