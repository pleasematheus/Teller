## 1. Workflow de CI (verificação automática)

- [x] 1.1 Criar `.github/workflows/ci.yml` com gatilhos `push` e `pull_request`
- [x] 1.2 Job em `windows-latest`: checkout, setup bun (`oven-sh/setup-bun`), `bun install --frozen-lockfile`
- [x] 1.3 Setup Rust estável (`dtolnay/rust-toolchain@stable`) com `rustfmt` e `clippy` + `Swatinem/rust-cache`
- [x] 1.4 Passo de checagem de tipos (`bun run build` ou `tsc --noEmit`) e `bun run test`
- [x] 1.5 Passos Rust: `cargo fmt --check`, `cargo clippy -- -D warnings`, `cargo test` em `src-tauri`
- [ ] 1.6 Validar que push/PR aciona o CI e reporta status (manual)

## 2. Workflow de build (gated)

- [x] 2.1 Criar `.github/workflows/build.yml` com gatilho `workflow_dispatch` apenas
- [x] 2.2 Adicionar input booleano `create_release` (default `false`) ao `workflow_dispatch`
- [x] 2.3 Job `verify` reusando os mesmos checks do CI (testes/tipos/lint)
- [x] 2.4 Job `bundle` com `needs: [verify]` e `environment: build-approval`
- [x] 2.5 No `bundle`: setup bun + Rust + cache, `bun install --frozen-lockfile`, `bun run tauri build`
- [x] 2.6 `actions/upload-artifact` dos instaladores em `src-tauri/target/release/bundle/{nsis,msi}/`
- [x] 2.7 Passo opcional de GitHub Release anexando instaladores quando `create_release == true`

## 3. Configuração do gate de aprovação (GitHub, manual)

- [x] 3.1 Criar Environment `build-approval` no repositório `pleasematheus/Teller`
- [x] 3.2 Configurar required reviewers na Environment `build-approval`
- [x] 3.3 (Manual) Disparar build de teste e confirmar que fica pendente até aprovação
- [x] 3.4 (Manual) Aprovar e confirmar geração + upload dos artefatos nsis e msi

## 4. Documentação

- [x] 4.1 Documentar no README/CONTRIBUTING como disparar o build manual (`gh workflow run build.yml` ou aba Actions) e quem pode aprovar
- [x] 4.2 Documentar comportamento esperado: push/PR nunca dispara build
