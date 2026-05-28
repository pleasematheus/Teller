## 1. Atualizar tipos e configuração

- [x] 1.1 Em `src/components/dashboard/types.ts`, substituir `"producao"` por `"relatorios"` no tipo `ModuleId`
- [x] 1.2 Em `src/components/dashboard/ModuleCard.tsx`, trocar `{ id: "producao", label: "Produção", icon: Factory01Icon }` por `{ id: "relatorios", label: "Relatórios", icon: Analytics02Icon }` no `MODULE_CONFIG`
- [x] 1.3 Remover o import de `Factory01Icon` e adicionar o import de `Analytics02Icon` de `@hugeicons/core-free-icons`

## 2. Atualizar roteamento no Dashboard

- [x] 2.1 Em `src/components/Dashboard.tsx`, atualizar qualquer referência a `"producao"` para `"relatorios"` no bloco de roteamento de módulos
