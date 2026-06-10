## Why

O backend confia 100% no frontend: nenhum command Tauri valida se há usuário autenticado. `change_user_password` troca a senha de qualquer usuário sem exigir a senha atual nem verificar quem chama; o mesmo vale para `toggle_user_active`, `create_user` e demais commands de escrita. O "login" existe apenas como estado do frontend. Qualquer código executando no webview (ex.: XSS) tem acesso administrativo total via IPC. Identificado em auditoria de segurança (2026-06-10).

## What Changes

- Login passa a gerar um token de sessão opaco no backend, retornado junto com os dados do usuário.
- Backend mantém estado de sessão (token -> usuário autenticado) em memória, gerenciado pelo Tauri.
- Commands sensíveis passam a exigir o token de sessão e retornam erro quando ausente/inválido. **BREAKING**: assinaturas dos commands ganham parâmetro `session_token`; frontend precisa propagá-lo.
- `change_user_password` passa a exigir a senha atual do usuário autenticado antes de trocar qualquer senha. **BREAKING**: novo parâmetro obrigatório.
- Novo command `logout` invalida o token de sessão.
- Commands de bootstrap (`check_setup`, `create_user` durante setup inicial, `login`, `find_user`, `check_business_setup`) permanecem públicos por necessidade; `create_user` pós-setup exige sessão.
- Frontend armazena o token em memória (estado React, nunca em storage persistente) e o injeta em cada `invoke` sensível.

## Capabilities

### New Capabilities
- `validacao-sessao`: gestão de sessão no backend — geração de token no login, validação de token em commands sensíveis, logout e expiração de sessão.

### Modified Capabilities
- `local-auth`: login passa a retornar token de sessão além dos dados do usuário; novo requisito de logout.
- `configuracoes-usuarios`: alteração de senha passa a exigir senha atual do usuário autenticado; criação de usuário pós-setup e ativação/desativação exigem sessão válida.

## Impact

- `src-tauri/src/auth.rs`: login gera token; novo command `logout`; validação de senha atual.
- `src-tauri/src/lib.rs`: novo estado gerenciado (`SessionState`); registro do command `logout`.
- `src-tauri/src/settings.rs`, `products.rs`, `stock.rs`, `sales.rs`, `financials.rs`, `reports.rs`: commands de escrita/leitura sensível ganham validação de sessão.
- Frontend (`src/`): contexto de autenticação passa a guardar token e injetá-lo nos `invoke`; fluxo de logout.
- Sem dependências novas no frontend. Backend pode precisar de fonte de aleatoriedade para o token (verificar se já existe via dependências atuais; caso contrário, pedir aprovação antes de adicionar crate).
- Testes: novos testes unitários Rust para geração/validação/invalidação de token; testes de senha atual em `change_user_password`.
