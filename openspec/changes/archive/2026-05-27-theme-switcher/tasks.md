## 1. Schema

- [x] 1.1 Adicionar tabela `User_Settings (user_id INTEGER NOT NULL REFERENCES "Users"(id), key TEXT NOT NULL, value TEXT NOT NULL, PRIMARY KEY (user_id, key))` ao final de `src/db/schema.sql`
- [x] 1.2 Criar função `ensure_user_settings_table(conn: &Connection) -> Result<(), String>` em `src-tauri/src/lib.rs` usando `CREATE TABLE IF NOT EXISTS`
- [x] 1.3 Chamar `ensure_user_settings_table` no início de `check_setup` (após `init_schema`), garantindo que a tabela existe em DBs novos e existentes

## 2. Backend — Tauri Commands

- [x] 2.1 Implementar `get_user_setting(state, user_id: i64, key: String) -> Result<Option<String>, String>` que faz SELECT em `User_Settings` e retorna `None` se não encontrado
- [x] 2.2 Implementar `set_user_setting(state, user_id: i64, key: String, value: String) -> Result<(), String>` que faz INSERT OR REPLACE em `User_Settings`
- [x] 2.3 Registrar `get_user_setting` e `set_user_setting` no `invoke_handler` em `lib.rs`
- [x] 2.4 Escrever testes unitários `#[cfg(test)]` para `get_user_setting` (key existente, key ausente) e `set_user_setting` (insert e update)
- [x] 2.5 Confirmar que `cargo test` passa sem erros

## 3. CSS — Paleta Light

- [x] 3.1 Definir paleta light em `:root` no `App.css` (fundos claros, texto escuro, mantendo accent amber)
- [x] 3.2 Manter `.dark` com a paleta escura existente (sem alterações)
- [x] 3.3 Remover `class="dark"` de `index.html`

## 4. Frontend — Tema Inicial Síncrono

- [x] 4.1 Em `src/main.tsx`, antes de `ReactDOM.createRoot`, ler `window.matchMedia('(prefers-color-scheme: dark)').matches` e adicionar `.dark` ao `document.documentElement` se o SO for dark

## 5. Frontend — Tipos e Utilitários

- [x] 5.1 Criar `src/types/theme.ts` com o tipo `Theme = 'light' | 'dark' | 'system'`
- [x] 5.2 Criar `src/lib/theme.ts` com funções puras: `applyTheme(theme: Theme, systemIsDark: boolean): void` e `resolveTheme(theme: Theme, systemIsDark: boolean): 'light' | 'dark'`
- [x] 5.3 Escrever testes em `src/lib/theme.test.ts` cobrindo todos os cenários de `resolveTheme` (light, dark, system×SO dark, system×SO light)

## 6. Frontend — Hook useTheme

- [x] 6.1 Criar `src/hooks/useTheme.ts` que recebe `user: AuthUser`, carrega a preferência via `get_user_setting(user.id, "theme")` no mount, e expõe `theme: Theme` e `setTheme(t: Theme)` (que chama `set_user_setting` e aplica o tema)
- [x] 6.2 Adicionar listener `matchMedia` ativo somente quando `theme === 'system'`, com cleanup no `useEffect`

## 7. Frontend — Componente ThemeToggle

- [x] 7.1 Criar `src/components/ThemeToggle.tsx` usando `DropdownMenu` do shadcn com as opções `Claro`, `Escuro` e `Padrão do sistema`
- [x] 7.2 Exibir indicador visual (ícone ou checkmark) na opção atualmente ativa

## 8. Integração

- [x] 8.1 Em `App.tsx`, no callback `onSuccess` do Login, após `setUser(u)`, chamar `get_user_setting(u.id, "theme")` e aplicar o tema antes de navegar para dashboard ou company-setup
- [x] 8.2 Adicionar `<ThemeToggle>` no header do Dashboard, ao lado do botão de logout, passando `user` como prop

## 9. Verificação Final

- [x] 9.1 Confirmar que `bun run test` passa com todos os testes de `theme.test.ts`
- [x] 9.2 Confirmar que `cargo test` passa com todos os testes do backend
- [x] 9.3 Confirmar que `bun run build` (ou `tauri build --debug`) compila sem erros de TypeScript
