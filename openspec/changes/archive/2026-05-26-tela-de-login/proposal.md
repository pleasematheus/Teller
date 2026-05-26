## Why

O app Teller não tem ponto de entrada protegido — qualquer abertura vai direto para o sistema. A tela de login é o primeiro passo para identificar quem está operando, garantindo rastreabilidade de todas as operações do mini-ERP por usuário.

## What Changes

- Adicionada tela de login com formulário (apelido/ID + senha)
- Adicionada tela de setup para criação do primeiro usuário (primeiro uso)
- Navegação state-based no `App.tsx` substituindo o template boilerplate
- Tauri commands Rust para inicialização do banco, criação de usuário, lookup leve de usuário e autenticação
- Schema `Users` já inclui `password_hash` (coluna adicionada manualmente)

## Capabilities

### New Capabilities

- `local-auth`: Autenticação local via DuckDB — criação de usuário no primeiro uso, login por apelido ou ID numérico com validação de senha hash
- `state-navigation`: Navegação entre telas via `useState` no `App.tsx` sem dependência de roteador externo

### Modified Capabilities

## Impact

- `src/App.tsx`: substituído inteiramente — remove boilerplate, implementa state machine de navegação
- `src-tauri/src/lib.rs`: novos Tauri commands (`check_setup`, `create_user`, `login`)
- `src/db/schema.sql`: já atualizado com `password_hash VARCHAR NOT NULL` em `Users`
- Nenhuma nova dependência JS necessária
- Rust: DuckDB já presente no `Cargo.toml` (`duckdb = "1.4.4"` com feature `bundled`)
