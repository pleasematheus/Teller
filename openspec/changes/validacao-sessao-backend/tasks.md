## 1. Aprovação e infraestrutura de sessão

- [x] 1.1 Obter aprovação explícita do usuário para adicionar `getrandom` como dependência direta em `src-tauri/Cargo.toml` (já presente como dep transitiva; zero crates novos compilados)
- [x] 1.2 Criar `SessionState(Mutex<Option<Session>>)` em `auth.rs` com `Session { token, user_id, nickname }` e registrar via `app.manage()` em `lib.rs`
- [x] 1.3 Implementar geração de token (32 bytes de `getrandom`, hex) e helper `require_session(&State<SessionState>, &str) -> Result<Session, String>` retornando `Err("Sessão inválida")`
- [x] 1.4 Testes unitários: token não-vazio, tokens distintos por login, novo login substitui sessão anterior, `require_session` recusa token vazio/inexistente/sem sessão

## 2. Commands de autenticação

- [x] 2.1 `login` passa a criar sessão e retornar `{ user, session_token }`; nenhum caminho de erro cria sessão
- [x] 2.2 Novo command `logout`: remove sessão ativa; sucesso sem efeito se não houver sessão; registrar em `lib.rs`
- [x] 2.3 `create_user`: público apenas quando `COUNT(Users WHERE active) == 0`; depois exige `require_session`
- [x] 2.4 `create_business_settings`: público apenas quando `Business_Settings` vazio; depois exige `require_session`
- [x] 2.5 `get_user_setting` / `set_user_setting` exigem `require_session`
- [x] 2.6 Testes unitários para 2.1 a 2.5 cobrindo os cenários das specs (login cria sessão, logout invalida, bootstrap público, pós-setup recusado)

## 3. change_user_password com re-autenticação

- [x] 3.1 Adicionar parâmetros `session_token` e `current_password`; validar sessão e verificar `current_password` contra o hash do usuário da sessão antes de alterar o alvo
- [x] 3.2 Testes unitários: senha atual correta altera alvo, senha atual incorreta recusa sem alterar, sem sessão recusa

## 4. Validação de sessão nos demais módulos

- [x] 4.1 `settings.rs`: `list_users`, `toggle_user_active`, payment methods, transaction fees, units of measure, system settings, business info/settings
- [x] 4.2 `products.rs`: todos os commands registrados
- [x] 4.3 `stock.rs`: todos os commands registrados
- [x] 4.4 `sales.rs`: todos os commands registrados
- [x] 4.5 `financials.rs`: todos os commands registrados
- [x] 4.6 `reports.rs`: `get_dre`
- [x] 4.7 Ao menos um teste unitário por módulo: command recusa token inválido com `Err("Sessão inválida")`
- [x] 4.8 Verificação exaustiva: conferir cada command de `generate_handler!` em `lib.rs` contra a lista pública permitida (`check_setup`, `check_business_setup`, `find_user`, `login`, `logout`); nenhum outro sem `require_session`

## 5. Frontend

- [x] 5.1 Contexto de autenticação guarda `sessionToken` em memória (nunca em localStorage/sessionStorage); tipo de retorno do `login` atualizado
- [x] 5.2 Wrapper central de `invoke` injeta `session_token` nos commands sensíveis; migrar todos os call sites (TypeScript acusa os faltantes)
- [x] 5.3 Fluxo de logout na UI: chama `logout`, limpa estado, volta à tela de login
- [x] 5.4 Formulário de alteração de senha ganha campo "senha atual"
- [x] 5.5 Atualizar/criar testes Vitest em `tests/lib/` para utilitários novos em `src/lib/` (ler arquivos antes de criar testes)

## 6. Verificação

- [x] 6.1 Spawnar subagente para rodar `cargo test` (src-tauri) e `bun run test --run` e reportar resultados
- [x] 6.2 `cargo clippy -- -D warnings` e `cargo fmt --check` limpos
- [x] 6.3 Teste manual (humano): fluxo completo setup -> login -> operações -> logout -> operações recusadas; alteração de senha exige senha atual; reload do webview exige novo login
