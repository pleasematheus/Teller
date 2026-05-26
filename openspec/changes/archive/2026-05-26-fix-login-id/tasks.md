## 1. Backend Rust (lib.rs)

- [x] 1.1 Em `check_setup`: chamar `init_schema(&conn)?` antes da contagem de usuários (substituindo a chamada a `init_users_table`)
- [x] 1.2 Em `login`: remover a linha `init_schema(&conn)?`
- [x] 1.3 Alterar a assinatura de `find_user` de `Result<bool, String>` para `Result<Option<String>, String>`
- [x] 1.4 Alterar o corpo de `find_user` para retornar `lookup_user(&conn, ...).map(|u| u.map(|row| row.nickname))`

## 2. Testes Rust

- [x] 2.1 Atualizar o teste `lookup_user_finds_by_nickname_case_insensitive` para verificar retorno `Some("João")` em vez de `true`
- [x] 2.2 Atualizar o teste `lookup_user_finds_by_numeric_id` para verificar retorno `Some("alice")` em vez de `true`
- [x] 2.3 Atualizar o teste `lookup_user_returns_none_for_unknown` para verificar retorno `None` em vez de `false`
- [x] 2.4 Executar `cargo test` e confirmar que todos os testes passam

## 3. Frontend (Login.tsx)

- [x] 3.1 Remover o tipo `UserFoundState` e o estado `userFound`
- [x] 3.2 Adicionar estado `resolvedNickname: string | null` (null = ainda não buscou) e `lookupError: boolean`
- [x] 3.3 Em `handleIdentifierChange`: resetar `resolvedNickname` para `null` e `lookupError` para `false`
- [x] 3.4 Em `handleIdentifierBlur`: chamar `invoke<string | null>("find_user", ...)` e, se retornar string, chamar `setIdentifier(nickname)` e `setResolvedNickname(nickname)`; se retornar `null`, setar `lookupError(true)`
- [x] 3.5 No JSX: substituir o bloco `{userFound === "not_found" && ...}` / `{userFound === "found" && ...}` por `{lookupError && <p className="text-teller-danger text-xs">Usuário não encontrado</p>}`
- [x] 3.6 No botão Submit: trocar `userFound === "not_found"` por `lookupError` na condição `disabled`
