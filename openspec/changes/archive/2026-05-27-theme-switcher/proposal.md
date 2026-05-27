## Why

O app não possui suporte a temas, ficando preso ao tema padrão do sistema ou ao tema fixo configurado no Tailwind. Oferecer ao usuário controle sobre o tema (claro, escuro ou padrão do sistema) é uma necessidade básica de usabilidade em aplicações desktop.

## What Changes

- Adiciona suporte a temas via classe CSS no `<html>` (abordagem padrão do Tailwind/shadcn)
- Aplica o tema do sistema sincronamente na inicialização do app (sem async, sem flash)
- Após o login, carrega e aplica a preferência salva do usuário logado
- Persiste a preferência **por usuário** em tabela `User_Settings` no DuckDB
- Cria botão/menu de alternância de tema acessível no layout do dashboard

## Capabilities

### New Capabilities

- `theme-management`: Controle de tema da aplicação — leitura/escrita da preferência por usuário no banco, aplicação da classe CSS no documento, e UI para alternar entre `light`, `dark` e `system`

### Modified Capabilities

- `company-setup`: A migração de schema precisa criar a tabela `User_Settings` se ainda não existir

## Impact

- **Frontend**: Novo componente `ThemeToggle`, novo hook `useTheme(user)`, lógica de tema do sistema em `main.tsx`, aplicação de preferência após login em `App.tsx`
- **Backend**: Novos Tauri commands `get_user_setting(user_id, key)` / `set_user_setting(user_id, key, value)` operando sobre tabela `User_Settings`
- **Schema**: Nova tabela `User_Settings (user_id INTEGER, key TEXT, value TEXT, PRIMARY KEY (user_id, key))`
- **Dependências**: Nenhuma nova dependência — Tailwind 4 e shadcn já suportam modo dark via classe
