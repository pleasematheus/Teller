## Context

O módulo Relatórios existe no Dashboard mas retorna apenas um placeholder. Os dados financeiros já estão distribuídos em três tabelas: `Financial_Entries` (receitas e despesas manuais), `Sales`/`Sales_Products` (vendas com preço e desconto) e `Sale_Payments` (recebimentos com taxas). Falta apenas (a) uma coluna de snapshot de custo em `Sales_Products`, (b) uma classificação de subtipo em `Financial_Entries` e (c) uma query que consolide tudo em 6 indicadores DRE.

ECharts 6.1.0 já está instalado. Stack: React 19, TypeScript, Tailwind 4, Tauri 2, Rust, DuckDB.

## Goals / Non-Goals

**Goals:**
- Exibir os 6 indicadores do mini-DRE para um mês selecionado
- Renderizar Sankey Chart mostrando o fluxo de receita até o resultado
- Classificar lançamentos financeiros por subtipo (despesa, custo, juros, imposto, devolucao)
- Capturar `cost_price` snapshot ao fechar uma venda

**Non-Goals:**
- Relatórios por período customizado (apenas mês por ora)
- Impostos sobre vendas deduzidos da receita bruta (fase posterior)
- DRE acumulado anual
- Exportação do DRE (PDF, CSV)

## Decisions

### 1. Migração de schema via `ensure_*` functions em `db.rs`

O projeto usa funções `ensure_*` em `db.rs` (ex: `ensure_unit_column`, `ensure_label_column`) para migrações incrementais via `ALTER TABLE ... ADD COLUMN IF NOT EXISTS`. Seguiremos o mesmo padrão com:
- `ensure_sales_products_cost_price()` 
- `ensure_financial_entries_expense_subtype()`

Alternativa considerada: `schema.sql` versionado — descartado porque o projeto não tem sistema de versão de migrations; o padrão `ensure_*` já está estabelecido.

### 2. Snapshot de `cost_price` capturado em `close_sale`

Ao fechar uma venda, o command `close_sale` já itera os `Sales_Products`. Nesse momento populamos `cost_price` com o valor atual de `Products.cost_price`. Vendas fechadas antes dessa mudança terão `cost_price = NULL` — o DRE exclui esses itens do CMV e exibe aviso.

Alternativa considerada: usar sempre `Products.cost_price` atual — descartado porque preços mudam e distorceria o CMV histórico.

### 3. `expense_subtype` como VARCHAR com valores controlados pela UI

Campo `expense_subtype VARCHAR` em `Financial_Entries`, null permitido. Valores válidos: `despesa | custo | juros | imposto | devolucao`. A validação é feita pela UI (select com opções fixas); o backend não rejeita outros valores para não bloquear dados legados.

Alternativa considerada: tipo ENUM no DuckDB — descartado porque DuckDB ENUM requer DROP/CREATE para alterar valores e complica migrações futuras.

### 4. Query DRE única no Rust com subconsultas DuckDB

O command `get_dre(month)` executa uma query SQL com CTEs que agrega:
- `receita_vendas`: `Sales_Products` onde a venda está `closed` no mês, com `price * quantity`
- `descontos`: diferença entre preço cheio e `discounted_price` + taxas de `Sale_Payments`
- `cmv`: `Sales_Products.cost_price * quantity` onde `cost_price IS NOT NULL`
- `financial_income`: `Financial_Entries` income do mês
- `despesas_custo`: `Financial_Entries` expense agrupadas por `expense_subtype` e `category`

Retorna struct `DreResult` com os 6 valores + `Vec<DreLineItem>` (category, subtype, amount) para o Sankey.

### 5. Transformação DRE → Sankey em função pura no frontend

`src/lib/dre.ts` exporta `dreToSankey(dre: DreResult): SankeyData`. Função pura, sem side effects, testável com Vitest. Se qualquer resultado intermediário for negativo (prejuízo), retorna `null` — o componente exibe fallback textual.

Alternativa considerada: gerar os nós/links no Rust — descartado porque a lógica de layout é responsabilidade da UI e o Rust não deve conhecer a estrutura do ECharts.

### 6. RelatoriosModule como shell com tela de seleção estilo Launcher

O `RelatoriosModule` tem duas camadas:
1. **Tela de seleção**: grid de cards, um por relatório disponível (padrão idêntico ao `Launcher` do Dashboard)
2. **View do relatório ativo**: renderiza o relatório selecionado com botão/seta de volta para a seleção

O estado interno é `activeReport: 'dre' | null`. Adicionar novos relatórios no futuro = adicionar um card na tela de seleção e um branch no render.

Alternativa considerada: tabs no topo — descartado porque tabs escalam mal com muitos relatórios; o padrão de cards é mais extensível e consistente com o Launcher existente.

Alternativa considerada: sidebar — descartado pelo mesmo motivo + adiciona complexidade de layout.

### 7. ECharts usado diretamente (sem wrapper)

Usar `echarts` via `useEffect` + `ref` no componente React, sem `echarts-for-react`. O projeto não usa wrappers de terceiros para outras libs; manter consistência.

## Risks / Trade-offs

- **CMV incompleto para vendas antigas** → Exibir badge "CMV parcial" no DRE quando existirem `Sales_Products` com `cost_price IS NULL` no período. Mitigação: documentar nas tasks.
- **Sankey não suporta resultado negativo** → Fallback: lista dos 6 indicadores com valores negativos destacados em vermelho, sem Sankey. Mitigação: implementar detecção de prejuízo em `dreToSankey`.
- **Categorias inconsistentes em `Financial_Entries` existentes** → O DRE agrupa por `category` (case-sensitive). Usuários com dados legados podem ver nós fragmentados no Sankey. Mitigação: normalizar `category` com `LOWER(TRIM(...))` na query.

## Migration Plan

1. Deploy das duas `ensure_*` functions (rodam no `setup` do Tauri, sem downtime)
2. A partir do deploy, `cost_price` é capturado em novas vendas fechadas
3. Dados históricos sem `cost_price`: DRE mostra CMV com aviso "dados parciais antes de [data do deploy]"
4. Rollback: as colunas adicionadas são nullable — nenhuma funcionalidade existente quebra se removidas
