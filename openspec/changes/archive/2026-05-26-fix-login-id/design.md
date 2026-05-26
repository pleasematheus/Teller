## Context

O app Tauri usa um único `Mutex<Connection>` para o banco DuckDB. O fluxo atual tem dois problemas:

1. **`init_schema` em `login`**: o comando Tauri `login` chama `execute_batch(SCHEMA)` com o schema.sql completo (14 tabelas, 13 sequences, 1 view) antes de autenticar. Na prática, `check_setup` já criou a tabela `Users` via `init_users_table`. Quando `login` tenta criar novamente os mesmos objetos (mesmo com `IF NOT EXISTS`), o DuckDB bundled 1.4.4 no Windows pode chamar `abort()` internamente, encerrando o processo.

2. **`find_user` retorna `bool`**: o `lookup_user` já busca o registro completo do usuário (incluindo `nickname`), mas `find_user` descarta tudo e devolve apenas se encontrou ou não.

## Goals / Non-Goals

**Goals:**
- Eliminar o crash ao logar com ID numérico movendo a inicialização do schema para o startup
- `find_user` retornar o `nickname` encontrado para o frontend substituir o campo
- `login` ficar com responsabilidade única: autenticar

**Non-Goals:**
- Alterar o fluxo de criação de usuário (Setup)
- Alterar a lógica de `lookup_user`
- Mudanças no Dashboard ou em outras telas

## Decisions

### D1 — `init_schema` vai para `check_setup`, não para um comando separado

`check_setup` já é o ponto de inicialização do banco no startup. Mover `init_schema` para lá garante que o schema completo exista antes de qualquer interação do usuário, sem adicionar uma nova chamada IPC.

Alternativa descartada: criar um comando `init_db` chamado no `useEffect` do App — adicionaria complexidade desnecessária e uma chamada de IPC extra.

### D2 — `find_user` retorna `Result<Option<String>, String>`

`Some(nickname)` quando encontrado, `None` quando não encontrado. O tipo Tauri serializa `Option<String>` como `string | null` no frontend, o que mapeia diretamente para o estado da UI.

Alternativa descartada: retornar um objeto `{ found: bool, nickname: string }` — desnecessariamente verboso para dois campos que se implicam.

### D3 — Estado `userFound` no frontend vira `string | null | "not_found"`

O componente Login troca o estado `UserFoundState ("idle" | "found" | "not_found")` por um estado que carrega o nickname: `string | null` onde `null` = não procurou ainda, `""` (string vazia especial) não é usado — usa-se um discriminador separado para erro.

Alternativa mais simples: dois estados — `resolvedNickname: string | null` e `lookupError: boolean`. Mais legível e elimina a union type explícita.

## Risks / Trade-offs

- **`init_schema` em `check_setup` roda na abertura do app**: no primeiro uso, cria todas as tabelas. Pode adicionar ~50-100ms ao startup em hardware lento. Aceitável para um app desktop local.
  → Mitigação: `IF NOT EXISTS` em todos os statements; executa uma vez e fica em cache pelo SO.

- **Substituição automática do campo pode surpreender o usuário**: se o usuário digitou "1" e o campo vira "joao", pode parecer um bug para quem não espera. 
  → Aceitável: é o comportamento explicitamente solicitado e resolve a ambiguidade de logar com ID.

## Migration Plan

1. Alterar `check_setup` em Rust para chamar `init_schema` antes da contagem de usuários
2. Remover `init_schema` de `login`
3. Alterar assinatura de `find_user` para `Result<Option<String>, String>`
4. Atualizar `Login.tsx`: estado, handler blur, JSX
5. Atualizar testes Rust para o novo retorno de `find_user`

Rollback: as mudanças são locais e sem migração de banco. Reverter o commit é suficiente.
