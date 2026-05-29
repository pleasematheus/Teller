## 1. Schema — Migrações de banco

- [x] 1.1 Adicionar função `ensure_sales_products_cost_price(conn)` em `src-tauri/src/db.rs` com `ALTER TABLE "Sales_Products" ADD COLUMN IF NOT EXISTS "cost_price" DECIMAL(10,2)`
- [x] 1.2 Adicionar função `ensure_financial_entries_expense_subtype(conn)` em `src-tauri/src/db.rs` com `ALTER TABLE "Financial_Entries" ADD COLUMN IF NOT EXISTS "expense_subtype" VARCHAR`
- [x] 1.3 Chamar ambas as funções no `setup` do Tauri em `src-tauri/src/lib.rs` (junto com as outras `ensure_*`)
- [x] 1.4 Atualizar `schema.sql` com as duas colunas para novos bancos
- [x] 1.5 Atualizar `test_db()` em `db.rs` para chamar as novas `ensure_*`

## 2. Backend — Snapshot de cost_price ao fechar venda

- [x] 2.1 Em `src-tauri/src/sales.rs`, no command `close_sale`, após fechar a venda iterar os `Sales_Products` e executar `UPDATE "Sales_Products" SET cost_price = (SELECT cost_price FROM "Products" WHERE id = product_id) WHERE sale_id = ?`
- [x] 2.2 Adicionar teste unitário em `sales.rs` verificando que `cost_price` é gravado ao fechar uma venda

## 3. Backend — Tipos e command get_dre

- [x] 3.1 Criar `src-tauri/src/reports.rs` com struct `DreResult` (6 campos f64: receita_bruta, receita_liquida, lucro_bruto, lucro_operacional, ebit, lucro_liquido) e `Vec<DreLineItem>` (category, subtype, amount)
- [x] 3.2 Implementar command `get_dre(month: String)` com query DuckDB usando CTEs: receita_vendas, financial_income, descontos, cmv, financial_expenses por subtype e category
- [x] 3.3 Incluir flag `cmv_partial: bool` em `DreResult` quando existirem `Sales_Products` com `cost_price IS NULL` no mês
- [x] 3.4 Normalizar `category` na query com `LOWER(TRIM(category))` para evitar fragmentação de nós no Sankey
- [x] 3.5 Adicionar teste unitário: mês sem dados retorna zeros
- [x] 3.6 Adicionar teste unitário: mês com vendas e despesas retorna indicadores corretos
- [x] 3.7 Adicionar teste unitário: mês com prejuízo (despesas > receitas) retorna valores negativos corretos
- [x] 3.8 Registrar `reports::get_dre` no `invoke_handler` em `lib.rs`

## 4. Backend — expense_subtype em Financial_Entries

- [x] 4.1 Adicionar campo `expense_subtype: Option<String>` à struct `FinancialEntry` em `src-tauri/src/financials.rs`
- [x] 4.2 Atualizar query `list_financial_entries` para incluir `expense_subtype` no SELECT
- [x] 4.3 Atualizar command `create_financial_entry` para receber e persistir `expense_subtype`
- [x] 4.4 Atualizar command `update_financial_entry` para receber e persistir `expense_subtype`

## 5. Frontend — Tipos TypeScript

- [x] 5.1 Adicionar `expense_subtype?: ExpenseSubtype | null` ao tipo `FinancialEntry` em `src/types/financeiro.ts`
- [x] 5.2 Criar tipo `ExpenseSubtype = 'despesa' | 'custo' | 'juros' | 'imposto' | 'devolucao'`
- [x] 5.3 Criar `src/types/relatorios.ts` com tipos `DreResult`, `DreLineItem` e `SankeyData` (nodes, links)

## 6. Frontend — Lógica pura DRE → Sankey

- [x] 6.1 Criar `src/lib/dre.ts` com função pura `dreToSankey(dre: DreResult): SankeyData | null` — retorna null se qualquer intermediário for ≤ 0
- [x] 6.2 Implementar geração de nós: fontes (Receita de Vendas, Outras Receitas se > 0), intermediários (Receita Bruta, Receita Líquida, Lucro Bruto, Lucro Operacional, EBIT) e folhas (Devoluções, Descontos, Taxas, CMV, cada categoria de despesa/custo, Juros, IR/CSLL, Lucro Líquido)
- [x] 6.3 Implementar geração de links com valores positivos corretos entre cada par de nós
- [x] 6.4 Criar `src/lib/dre.test.ts` com testes Vitest: mês com lucro gera Sankey válido, mês com prejuízo retorna null, Receita Bruta zero retorna null, categorias múltiplas geram nós separados

## 7. Frontend — Formulário Financial Entry com expense_subtype

- [x] 7.1 Em `src/components/financeiro/EntradaForm.tsx`, adicionar select "Tipo de despesa" visível apenas quando `entryType === 'expense'`
- [x] 7.2 Opções do select: vazia (sem classificação), Despesa, Custo, Juros, Imposto, Devolução
- [x] 7.3 Passar `expense_subtype` nos invokes `create_financial_entry` e `update_financial_entry`
- [x] 7.4 Pré-preencher `expense_subtype` ao editar lançamento existente

## 8. Frontend — RelatoriosModule shell (tela de seleção)

- [x] 8.1 Criar `src/components/relatorios/RelatoriosModule.tsx` com estado `activeReport: 'dre' | null` e tela inicial exibindo grid de cards (padrão do `Launcher`)
- [x] 8.2 Criar card "Mini-DRE" na tela de seleção com título, descrição curta e ícone
- [x] 8.3 Ao clicar no card, transicionar para a view do relatório ativo; exibir botão/seta "Voltar" para retornar à seleção
- [x] 8.4 Criar `src/components/relatorios/index.ts` exportando `RelatoriosModule`
- [x] 8.5 Substituir o placeholder de Relatórios em `src/components/Dashboard.tsx` pelo `RelatoriosModule`

## 9. Frontend — View do Mini-DRE

- [x] 9.1 Criar `src/components/relatorios/DreView.tsx` com seletor de mês (reutilizar padrão do FinanceiroModule: setas + MonthPicker)
- [x] 9.2 Implementar tabela DRE com 6 linhas: nome do indicador, valor em R$, percentual em relação à Receita Bruta (ou `—` se zero)
- [x] 9.3 Destacar valores negativos em vermelho na tabela
- [x] 9.4 Exibir badge "CMV parcial" quando `dre.cmv_partial === true`
- [x] 9.5 Criar `src/components/relatorios/SankeyChart.tsx` que recebe `SankeyData | null` e renderiza o gráfico ECharts via `useEffect` + `ref`; exibe mensagem de fallback quando `null`
- [x] 9.6 Compor `DreView` com tabela DRE acima e `SankeyChart` abaixo
