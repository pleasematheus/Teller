## Context

Auditoria de segurança (2026-06-10) constatou que nenhum command Tauri valida autenticação. O login (`auth::login`) apenas verifica credenciais e devolve dados do usuário; a partir daí o backend aceita qualquer chamada IPC, incluindo `change_user_password` para qualquer usuário sem senha atual. O app é desktop, single-window, mono-máquina, com usuários locais em DuckDB e hash bcrypt. Não há roles: todo usuário ativo tem acesso total às configurações (modelo atual da UI).

Estado atual relevante:

- `lib.rs` gerencia `DbState(Mutex<Connection>)` e registra ~50 commands.
- `auth.rs` tem `login`, `create_user`, `check_setup`, `find_user`, settings por usuário.
- Frontend mantém o usuário logado apenas em estado React; não persiste credenciais.

## Goals / Non-Goals

**Goals:**

- Commands sensíveis recusam chamadas sem sessão válida.
- `change_user_password` exige senha atual do usuário autenticado (re-autenticação).
- Token de sessão opaco, gerado no backend, vivendo apenas em memória (backend e frontend).
- Logout explícito invalida a sessão.

**Non-Goals:**

- Roles/permissões por usuário (não existem hoje; fora de escopo).
- Expiração por tempo/inatividade: a sessão vive até logout, novo login ou encerramento do processo.
- Persistência de sessão entre execuções do app.
- Derivar `user_id` dos commands `get_user_setting`/`set_user_setting` a partir da sessão (mantêm o parâmetro `user_id`, mas passam a exigir sessão válida).

## Decisions

1. **Sessão única em memória: `SessionState(Mutex<Option<Session>>)`**
   `Session { token: String, user_id: i64, nickname: String }`, gerenciada via `app.manage()`.
   Alternativa considerada: `HashMap<token, Session>` multi-sessão. Rejeitada: app desktop single-window tem no máximo um usuário logado; novo login substitui a sessão anterior (invalida o token antigo de graça).

2. **Token: 32 bytes aleatórios do SO, hex-encoded**
   Gerado com `getrandom` (já presente na árvore de dependências como dep transitiva do bcrypt; adicioná-la como dependência direta não compila nenhum crate novo). Requer aprovação do usuário antes do apply, conforme regra do projeto sobre dependências.
   Alternativa: `uuid` v4 — também transitiva, mas API maior para a mesma necessidade.
   Comparação de token por igualdade simples: o canal é IPC local, sem oráculo de timing exposto pela rede.

3. **Helper único de validação em `auth.rs`**
   `pub(crate) fn require_session(state: &State<SessionState>, token: &str) -> Result<Session, String>` — retorna erro "Sessão inválida" genérico. Cada command sensível ganha parâmetros `session_token: String` e `session: State<SessionState>` e chama o helper na primeira linha. Padrão mecânico, fácil de revisar e de testar.

4. **Bootstrap permanece público com guarda de estado**
   - `check_setup`, `check_business_setup`, `login`, `find_user`: públicos (necessários antes de autenticar).
   - `create_user`: público apenas enquanto `COUNT(Users WHERE active) == 0` (setup inicial); depois exige sessão.
   - `create_business_settings`: público apenas enquanto `Business_Settings` vazio; depois exige sessão.
   Alternativa: command separado para setup. Rejeitada: duplicaria lógica e quebraria o frontend sem ganho.

5. **`change_user_password` re-autentica o chamador**
   Novo parâmetro `current_password: String`, verificado via bcrypt contra o hash do usuário da sessão (não do usuário alvo). Mantém o caso de uso "operador A reseta senha do operador B" sem introduzir roles, mas garante que quem chama provou conhecer a própria senha agora.

6. **Frontend: token em memória React, nunca em storage**
   O contexto de autenticação guarda `{ user, sessionToken }`; um wrapper de `invoke` injeta `sessionToken` nos commands sensíveis. Logout limpa o estado e chama o command `logout`.

7. **Login retorna `{ user, session_token }`**
   **BREAKING** para o frontend: struct de retorno muda. Adaptação no mesmo change.

## Risks / Trade-offs

- [Sessão única] Dois logins simultâneos impossíveis → aceitável: app single-window; novo login derruba o anterior.
- [~50 commands para alterar] Risco de esquecer um command sensível → mitigação: checklist exaustiva em tasks.md derivada de `lib.rs::generate_handler!`; teste unitário para cada categoria; revisão final por grep de commands sem `require_session`.
- [Quebra do frontend em massa] Cada `invoke` sensível precisa do token → mitigação: wrapper central de `invoke`; TypeScript aponta os call sites faltantes na compilação.
- [XSS ainda rouba o token em memória] Validação de sessão reduz, não elimina, o impacto de XSS → mitigação complementar já aplicada: CSP restritiva (2026-06-10). Defesa em profundidade.
- [Token some se o webview recarregar (F5 em dev)] Usuário precisa logar de novo → aceitável; comportamento igual ao de fechar o app.

## Migration Plan

1. Backend: `SessionState` + token no login + `logout` + helper (`auth.rs`, `lib.rs`).
2. Backend: aplicar `require_session` módulo a módulo (settings, products, stock, sales, financials, reports, auth user-settings), com testes por módulo.
3. Frontend: contexto de auth guarda token; wrapper de invoke; fluxo de logout.
4. Verificação final: grep por commands registrados sem validação; `cargo test`; `bun run test`; teste manual delegado.

Rollback: revert do change inteiro (sem migração de banco — nada persiste).

## Open Questions

- Adicionar `getrandom` como dependência direta no `Cargo.toml` precisa de aprovação explícita do usuário (regra do projeto). Alternativa zero-dependência: ler `SystemTime` + endereço de heap como seed é INACEITÁVEL para token de segurança; se a aprovação for negada, considerar expor salt aleatório do bcrypt — não recomendado.
