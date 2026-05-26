## Why

Com o fluxo de autenticação funcionando, o estado `"dashboard"` no `App.tsx` ainda mostra apenas um placeholder de texto. É necessário uma tela inicial mínima para validar o fluxo completo (login → dashboard) e servir de base para as funcionalidades futuras do mini-ERP.

## What Changes

- Substituído o placeholder de dashboard por um componente `<Dashboard />` real
- Tela exibe saudação com nome do usuário logado e estrutura básica de layout (header + área de conteúdo)
- Botão de logout que retorna à tela de login

## Capabilities

### New Capabilities

- `dashboard`: Tela inicial pós-login com identidade do usuário e estrutura de layout para expansão futura

### Modified Capabilities

- `state-navigation`: Adicionada transição de `"dashboard"` de volta para `"login"` via logout

## Impact

- `src/App.tsx`: substitui placeholder pelo componente `<Dashboard />`; passa callback `onLogout`
- `src/components/Dashboard.tsx`: novo componente
- `openspec/specs/state-navigation/spec.md`: nova transição de logout
