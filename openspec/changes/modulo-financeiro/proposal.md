## Why

A tabela `Financial_Entries` existe no schema desde o início mas não tem UI — despesas fixas, variáveis e receitas não-operacionais não podem ser registradas, tornando o DRE futuro inútil sem dados. O módulo Financeiro fecha essa lacuna com uma interface completa de lançamentos.

## What Changes

- Adicionar `"financeiro"` como novo `ModuleId` (7º módulo no dashboard)
- Adicionar card "Financeiro" no launcher com ícone de finanças
- Criar componente `FinanceiroModule` com:
  - Tabs "Receitas" | "Despesas"
  - Seletor de mês (ShadCN Calendar, já instalado)
  - Seção de sugestões recorrentes (entradas `is_recurring=true` sem correspondente no mês atual)
  - Lista de lançamentos do mês filtrado
  - Formulário de criação/edição inline ou em sheet
- Adicionar 5 comandos Tauri em `lib.rs`: `list_financial_entries`, `create_financial_entry`, `update_financial_entry`, `delete_financial_entry`, `list_recurring_suggestions`
- Ajustar o grid do launcher de 3 colunas para acomodar 7 módulos

## Capabilities

### New Capabilities

- `gestao-financeira`: CRUD de lançamentos financeiros (receitas e despesas) com filtro por mês, sugestões recorrentes e separação fixo/variável

### Modified Capabilities

- `dashboard`: novo módulo `"financeiro"` adicionado ao `ModuleId` e ao `MODULE_CONFIG`

## Impact

- `src/components/dashboard/types.ts` — `ModuleId` ganha `"financeiro"`
- `src/components/dashboard/ModuleCard.tsx` — `MODULE_CONFIG` ganha entrada Financeiro
- `src/components/dashboard/Launcher.tsx` — ajuste de grid (3 cols → 4 cols ou 3+wrap)
- `src/components/Dashboard.tsx` — roteamento para `FinanceiroModule`
- `src/components/financeiro/` — novos arquivos de componente
- `src-tauri/src/lib.rs` — 5 novos Tauri commands
- Sem novas dependências (Calendar do ShadCN já instalado, DuckDB já em uso)
