## 1. Dashboard — registrar novo módulo

- [x] 1.1 Em `src/components/dashboard/types.ts`, adicionar `"financeiro"` ao tipo `ModuleId`
- [x] 1.2 Em `src/components/dashboard/ModuleCard.tsx`, adicionar `{ id: "financeiro", label: "Financeiro", icon: Wallet01Icon }` ao `MODULE_CONFIG` e importar `Wallet01Icon` de `@hugeicons/core-free-icons`
- [x] 1.3 Em `src/components/Dashboard.tsx`, adicionar roteamento `if (module === "financeiro") return <FinanceiroModule />`

## 2. Backend — Tauri commands (Rust)

- [x] 2.1 Adicionar struct `FinancialEntry` com todos os campos serializáveis em `lib.rs`
- [x] 2.2 Implementar `list_financial_entries(month: String, entry_type: String)` — filtra por `strftime('%Y-%m', reference_date)` e `type`
- [x] 2.3 Implementar `list_recurring_suggestions(month: String, entry_type: String)` — query de lacunas com `DISTINCT ON (name, type)`
- [x] 2.4 Implementar `create_financial_entry(name, category, amount, entry_type, is_fixed, is_recurring, reference_date)` com validação de campos obrigatórios
- [x] 2.5 Implementar `update_financial_entry(id, name, category, amount, is_fixed, is_recurring, reference_date)` com verificação de existência
- [x] 2.6 Implementar `delete_financial_entry(id)` com verificação de existência
- [x] 2.7 Registrar os 5 novos commands no `invoke_handler!` em `lib.rs`
- [x] 2.8 Escrever testes unitários `#[cfg(test)]` para criação, listagem e sugestões recorrentes

## 3. Frontend — estrutura de arquivos

- [x] 3.1 Criar `src/components/financeiro/index.ts` exportando `FinanceiroModule`
- [x] 3.2 Criar `src/components/financeiro/FinanceiroModule.tsx` com tabs "Receitas" | "Despesas" e estado de mês selecionado
- [x] 3.3 Criar `src/components/financeiro/EntradaList.tsx` — lista de lançamentos do mês com ações de editar e excluir
- [x] 3.4 Criar `src/components/financeiro/EntradaForm.tsx` — sheet lateral com formulário (nome, categoria, valor, data, is_fixed, is_recurring)
- [x] 3.5 Criar `src/components/financeiro/SugestoesRecorrentes.tsx` — seção de sugestões com botão "Registrar"

## 4. Frontend — integração e comportamento

- [x] 4.1 Integrar seletor de mês (ShadCN Calendar) no `FinanceiroModule`, inicializando com o mês corrente
- [x] 4.2 Conectar `EntradaList` ao command `list_financial_entries` reagindo a mudanças de mês e tab
- [x] 4.3 Conectar `SugestoesRecorrentes` ao command `list_recurring_suggestions`; ao clicar "Registrar", abrir `EntradaForm` pré-preenchido
- [x] 4.4 Conectar `EntradaForm` aos commands `create_financial_entry` e `update_financial_entry`; invalidar lista após salvar
- [x] 4.5 Implementar exclusão com confirmação simples conectada ao command `delete_financial_entry`
- [x] 4.6 Exibir indicador visual de "Fixo" nos cards da lista (badge ou ícone)
