## Context

O app atualmente usa tema escuro fixo: `index.html` tem `class="dark"` hardcoded no `<html>`, e `App.css` define `@custom-variant dark (&:is(.dark *))` com todas as variáveis CSS de cor apenas para dark. Não existe paleta light — `:root` e `.dark` têm os mesmos valores (ambos dark). A feature requer reestruturar o CSS antes de qualquer outra coisa.

O banco usa DuckDB com `init_schema` que pula a criação se `Production_Inputs` já existir — qualquer nova tabela precisa de tratamento especial para DBs existentes.

## Goals / Non-Goals

**Goals:**
- Permitir ao usuário escolher entre `light`, `dark` e `system` (padrão do sistema)
- Persistir a preferência **por usuário** no banco DuckDB
- Aplicar o tema do sistema imediatamente na inicialização (sem async, sem flash)
- Aplicar a preferência do usuário logo após o login
- Botão de toggle acessível no Dashboard

**Non-Goals:**
- Temas customizados além de light/dark
- Preferência global por instalação (independente do usuário logado)
- Suporte a múltiplos monitores com esquemas de cor diferentes

## Decisions

### 1. Armazenamento: `User_Settings (user_id, key, value)` — por usuário

Tabela `User_Settings (user_id INTEGER NOT NULL REFERENCES "Users"(id), key TEXT NOT NULL, value TEXT NOT NULL, PRIMARY KEY (user_id, key))`. Cada usuário tem suas próprias preferências. Consistente com a arquitetura existente (tudo no DuckDB).

**Alternativas descartadas:**
- `App_Settings` global: um único tema por dispositivo ignora múltiplos usuários
- `localStorage`: não segue o padrão do projeto; seria uma segunda fonte de verdade
- Coluna `theme` em `Users`: acoplaria a tabela de autenticação a preferências de UI

### 2. Criação da tabela: função `ensure_user_settings_table` separada de `init_schema`

`init_schema` pula inteiramente se o DB já existir. Para cobrir DBs existentes, chamamos `ensure_user_settings_table(conn)` a cada startup — ela faz `CREATE TABLE IF NOT EXISTS` de forma segura e idempotente.

### 3. Sequência de carregamento em dois tempos

**Tempo 1 — Sincronamente em `main.tsx`**, antes do React renderizar: lê `window.matchMedia('(prefers-color-scheme: dark)').matches` e aplica `.dark` no `<html>` imediatamente. Zero async, zero flash. Esse é o estado inicial.

**Tempo 2 — Após o login em `App.tsx`**: chama `get_user_setting(user.id, "theme")` e aplica a preferência salva, sobrescrevendo o estado inicial se necessário. Como o login já envolve uma transição de tela, qualquer ajuste de tema é imperceptível.

**Alternativa descartada:** carregar preferência na SplashScreen (antes do login) — não funciona com preferência per-user porque ainda não se sabe quem está logando.

### 4. CSS: separar paleta light em `:root`, dark apenas em `.dark`

Estado atual: `:root` e `.dark` têm as mesmas variáveis dark. Precisamos:
- `:root` → paleta light (fundos claros, texto escuro)
- `.dark` → paleta dark existente (sem mudança)
- Remover `class="dark"` do `index.html` (o JS aplicará conforme o SO na inicialização)

A paleta light seguirá o padrão shadcn/slate complementar ao dark existente.

### 5. Aplicação do tema: classe `.dark` no `<html>`

Padrão Tailwind + shadcn. `system` → lê `matchMedia`. `light` → remove `.dark`. `dark` → adiciona `.dark`. O listener de mudança de sistema só é ativo quando o modo é `system`.

### 6. UI: `ThemeToggle` como dropdown com 3 opções no header do Dashboard

Um `DropdownMenu` do shadcn com ícone `SunMoon` (ou similar) que lista Claro / Escuro / Padrão do sistema. Fica no canto superior direito do Dashboard, junto ao botão de logout.

## Risks / Trade-offs

- **Troca de tema ao fazer login** → Se o SO usa dark mas o usuário salvou `light`, haverá uma transição visível no momento do login. Aceitável: a transição acompanha a mudança de tela (login → dashboard) e é natural para o usuário.
- **DB existente sem `User_Settings`** → Coberto pela `ensure_user_settings_table` idempotente chamada no startup.
- **Sistema muda tema enquanto app está aberto (modo `system`)** → Coberto com `addEventListener` no `matchMedia`, removido ao mudar de modo.

## Migration Plan

1. Adicionar `User_Settings` ao `schema.sql` (para DBs novos)
2. Adicionar `ensure_user_settings_table` chamada no startup do backend
3. Reestruturar `App.css` com paleta light em `:root`
4. Remover `class="dark"` de `index.html`
5. Aplicar tema do sistema sincronamente em `main.tsx`
6. Implementar Tauri commands, hook `useTheme`, componente `ThemeToggle`
7. Aplicar preferência do usuário no callback de login em `App.tsx`

**Rollback:** reverter `index.html` para `class="dark"` e remover o hook/componente restaura o comportamento anterior imediatamente.

## Open Questions

- Nenhuma — todas as decisões foram tomadas com base na arquitetura existente.
