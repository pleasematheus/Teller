## 1. Schema e migração de banco

- [x] 1.1 Adicionar coluna `sale_id` INTEGER (NULL) e FK `FOREIGN KEY ("sale_id") REFERENCES "Sales" ("id")` na definição de `Stock_Movements` em `src/db/schema.sql` (bloco `Stock_Movements` + view `Stock` reordenados para depois de `Sales`, pois DuckDB exige a tabela alvo da FK criada antes)
- [x] 1.2 Apagar o arquivo de banco DuckDB local de desenvolvimento (versão antiga) (nenhum DB existente em `%APPDATA%\com.teller.app` - nada a apagar)
- [ ] 1.3 Recriar o banco rodando o app para aplicar o novo schema (automático no próximo launch, pois não há DB antigo; `init_schema` cria com o novo schema)

## 2. Backend - fechamento da venda

- [x] 2.1 Em `src-tauri/src/sales.rs` (`close_sale`), incluir `sale_id` no INSERT de `Stock_Movements` do caminho de produto sem componentes
- [x] 2.2 Em `src-tauri/src/sales.rs` (`close_sale`), incluir `sale_id` no INSERT de `Stock_Movements` do caminho de insumos via BOM
- [x] 2.3 Adicionar teste em `sales.rs` validando que a baixa de produto sem componentes grava `sale_id` igual ao id da venda
- [x] 2.4 Adicionar teste em `sales.rs` validando que as baixas de insumos via BOM gravam `sale_id` igual ao id da venda

## 3. Backend - leitura do histórico

- [x] 3.1 Adicionar campo `sale_id: Option<i64>` à struct `StockMovement` em `src-tauri/src/stock.rs`
- [x] 3.2 Incluir `sale_id` no SELECT e no mapeamento de `list_stock_movements`
- [x] 3.3 Adicionar teste em `stock.rs` validando que `list_stock_movements` retorna `sale_id` preenchido para venda e `None` para compra/perda/ajuste

## 4. Frontend

- [x] 4.1 Adicionar `sale_id: number | null` à interface `StockMovement` em `src/types/stock.ts`
- [x] 4.2 Em `src/components/stock/MovementHistory.tsx`, adicionar coluna "Venda" no cabeçalho e na linha, exibindo `#<sale_id>` quando presente e vazio quando `null`

## 5. Verificação

- [x] 5.1 Rodar `cargo test` (backend) e confirmar tudo passando (123 passed; 0 failed)
- [x] 5.2 Rodar `bun run test` (frontend) e confirmar tudo passando (60 passed; erros tsc pré-existentes em EntradaForm/calendar/spinner, não relacionados)
- [x] 5.3 [Manual - testador humano] Fechar uma venda, abrir a tela de estoque, selecionar um insumo consumido e confirmar que o ID da venda aparece no histórico
- [x] 5.4 [Manual - testador humano] Confirmar que movimentações de compra/perda/ajuste exibem o campo de venda vazio
