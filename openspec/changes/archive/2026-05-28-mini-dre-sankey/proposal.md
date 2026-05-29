## Why

O módulo de Relatórios existe no Dashboard mas exibe apenas um placeholder. O primeiro relatório real é um mini-DRE com visualização Sankey, permitindo que o usuário entenda em um único painel como a receita bruta se transforma em lucro líquido ao longo de um mês.

## What Changes

- Substituir o placeholder do módulo Relatórios por um `RelatoriosModule` funcional com seletor de mês, tabela DRE e Sankey Chart
- Adicionar coluna `cost_price` em `Sales_Products` — snapshot do custo do produto no momento do fechamento da venda
- Adicionar coluna `expense_subtype` em `Financial_Entries` — classifica despesas em `despesa | custo | juros | imposto | devolucao`
- Novo Tauri command `get_dre(month)` retornando os 6 indicadores e breakdown por categoria
- Capturar `cost_price` automaticamente ao fechar uma venda (`close_sale`)
- Campo `expense_subtype` no formulário de lançamento financeiro (visível apenas para despesas)

## Capabilities

### New Capabilities

- `relatorio-dre`: Mini-DRE mensal com 6 indicadores (Receita Bruta → Receita Líquida → Lucro Bruto → Lucro Operacional → EBIT → Lucro Líquido) e Sankey Chart mostrando o fluxo de receita até o resultado

### Modified Capabilities

- `gestao-financeira`: Lançamentos financeiros ganham campo `expense_subtype` que classifica o tipo de despesa — altera o formulário de cadastro e a estrutura de dados retornada

## Impact

- **Schema**: `Sales_Products` + `cost_price`; `Financial_Entries` + `expense_subtype`; migrações via `ALTER TABLE ... ADD COLUMN IF NOT EXISTS`
- **Rust**: novo módulo `reports.rs` com command `get_dre`; `sales.rs` alterado para capturar `cost_price` em `close_sale`; `financials.rs` atualizado para ler/gravar `expense_subtype`
- **Frontend**: novo componente `RelatoriosModule`; `EntradaForm` atualizado com select de subtipo; `src/lib/dre.ts` com função pura de transformação DRE → nós/links Sankey
- **Dependências**: ECharts já instalado — sem novas dependências
