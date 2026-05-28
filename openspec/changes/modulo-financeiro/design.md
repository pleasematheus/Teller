## Context

O Teller já tem DuckDB com a tabela `Financial_Entries` no schema, mas sem nenhuma UI ou comando Tauri para manipulá-la. O módulo Financeiro introduz o CRUD completo dessa tabela, exposto como o 7º módulo no dashboard. O Calendar do ShadCN já está instalado. Todos os outros padrões (Tauri commands, componentes React com sub-tabs inline) estão estabelecidos nos módulos existentes.

## Goals / Non-Goals

**Goals:**
- CRUD completo de `Financial_Entries` (receitas e despesas)
- Separação por tab (Receitas | Despesas) e filtro por mês
- Sugestões de lançamentos recorrentes não registrados no mês atual
- Formulário com campos: nome, categoria, valor, data de referência, fixo?, recorrente?
- 5 novos Tauri commands em Rust com testes unitários

**Non-Goals:**
- Lógica de agendamento automático de recorrências (sugestões são passivas)
- Campos avançados do schema: `quantity`, `unit_cost`, `product_id`, `due_date`, `description`
- Relatórios ou gráficos (pertencem ao módulo Relatórios futuro)
- Conciliação bancária ou importação de extratos

## Decisions

**Filtro de mês como `YYYY-MM` string**
O comando `list_financial_entries` recebe `month: String` no formato `"2026-05"`. No Rust, a query filtra com `strftime('%Y-%m', reference_date) = ?`. Alternativa considerada: passar ano e mês separados — rejeitado por ser mais verboso sem ganho.

**Sugestões via query de lacunas**
`list_recurring_suggestions(month)` busca entradas `is_recurring=true` de qualquer mês anterior cujo `name+type` não apareça em nenhuma entrada com `reference_date` no mês alvo. Isso é declarativo e não requer tabela extra de "agendamentos".

```sql
SELECT DISTINCT ON (name, type) name, category, type, amount, is_fixed
FROM "Financial_Entries"
WHERE is_recurring = true
  AND strftime('%Y-%m', reference_date) < ?
  AND (name, type) NOT IN (
    SELECT name, type FROM "Financial_Entries"
    WHERE strftime('%Y-%m', reference_date) = ?
  )
ORDER BY name, type, reference_date DESC
```

**Grid do launcher: 3+wrap → `grid-cols-3` com 7 itens**
7 módulos em `grid-cols-3` resulta em 2 linhas (3+4 ou 3+3+1). Mantém o padrão atual sem refatoração de layout. Alternativa (4 colunas) considerada — rejeitada por quebrar a simetria visual com janela estreita.

**Estrutura de arquivos**
Seguir o padrão dos módulos existentes:
```
src/components/financeiro/
  FinanceiroModule.tsx   ← componente raiz com tabs e filtro de mês
  EntradaForm.tsx        ← formulário de criação/edição
  EntradaList.tsx        ← lista de lançamentos do mês
  SugestoesRecorrentes.tsx ← seção de sugestões
  index.ts
```

**Formulário inline vs sheet**
Usar sheet (painel lateral deslizante) como nos módulos de Vendas e Compras — consistente com o padrão já estabelecido no projeto.

## Risks / Trade-offs

[Performance] `DISTINCT ON` não é SQL padrão — DuckDB suporta, mas a query de sugestões pode ser lenta com muitos anos de histórico → Mitigação: a query já tem `WHERE reference_date < mês_atual`, índice natural pelo campo de data.

[UX] O campo `reference_date` pode confundir com "data de vencimento" (`due_date`) → Mitigação: rotular claramente no formulário como "Competência (mês de referência)".

[Grid] 7 módulos em grid-cols-3 resulta em linha final com apenas 1 card → Aceitável por ora; a reorganização do grid pode ser feita junto com a implementação do módulo Relatórios.
