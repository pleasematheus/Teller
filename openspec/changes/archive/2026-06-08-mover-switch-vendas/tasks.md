## 1. Nova secao Vendas

- [x] 1.1 Criar `src/components/configuracoes/VendasSection.tsx` movendo a logica do switch `allow_negative_stock_sale` de `SistemaSection` (estado, `useEffect` de leitura, `handleToggle`, JSX do card)
- [x] 1.2 Em `ConfiguracoesModule.tsx`, adicionar `"vendas"` ao tipo `Section`
- [x] 1.3 Adicionar item `{ id: "vendas", label: "Vendas" }` em `NAV_ITEMS` imediatamente ANTES de `{ id: "sistema", ... }`
- [x] 1.4 Importar `VendasSection` e adicionar `{section === "vendas" && <VendasSection />}` no `<main>`

## 2. Limpar SistemaSection

- [x] 2.1 Remover de `SistemaSection.tsx` o switch e a logica de `allow_negative_stock_sale` (estado, `useEffect`, `handleToggle`, import de `parseSystemSettingBool`/`Switch` se nao mais usados)
- [x] 2.2 Manter cabecalho ("Sistema" + descricao) como placeholder ate a mudanca de bandeja

## 3. Testes

- [x] 3.1 Rodar `bun run test` e confirmar que testes de `parseSystemSettingBool`/`systemSettings` continuam passando (logica inalterada)
- [x] 3.2 Rodar `bun run build` / type check sem erros
- [x] 3.3 MANUAL (testador humano): abrir Configuracoes, confirmar nova aba "Vendas" antes de "Sistema", alternar o switch e confirmar persistencia ao reabrir

## 4. Spec

- [x] 4.1 Validar `openspec validate mover-switch-vendas`
