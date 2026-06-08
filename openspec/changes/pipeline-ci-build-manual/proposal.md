## Why

O projeto não possui pipeline de CI/CD. Hoje testes (cargo + vitest), lint e build do instalador Windows dependem de execução manual na máquina do dev, sem garantia de que `main` esteja sempre verde. É necessário automatizar verificação contínua, mas sem disparar builds pesados do bundle Tauri automaticamente: o build (e eventual release) só deve ocorrer quando o mantenedor autorizar explicitamente.

## What Changes

- Adicionar workflow de CI que roda automaticamente em push e pull request: instala dependências (bun + Rust toolchain), roda `bun run test` (Vitest) e `cargo test`, valida tipos (`tsc`) e checagens Rust (`cargo fmt --check`, `cargo clippy`).
- Adicionar job/workflow de build do bundle Tauri Windows (nsis + msi) que NÃO roda automaticamente: só é disparado manualmente via `workflow_dispatch` e/ou bloqueado por uma GitHub Environment com regra de aprovação manual (required reviewers).
- O job de build depende do sucesso do CI (lint + testes) antes de poder rodar.
- Publicação opcional dos artefatos de build (instaladores) como artifacts do workflow, e opcionalmente como GitHub Release quando o disparo manual indicar release.
- Documentar como autorizar/disparar o build manualmente.

## Capabilities

### New Capabilities
- `pipeline-ci-cd`: define o comportamento da pipeline de integração e entrega contínua: gatilhos automáticos de verificação (testes/lint/tipos), gate manual de autorização para build, geração de artefatos do instalador Windows e dependência do build sobre o CI verde.

### Modified Capabilities
<!-- Nenhuma capability de comportamento existente muda os requisitos. -->

## Impact

- Novos arquivos em `.github/workflows/` (ex.: `ci.yml`, `build.yml`).
- Configuração de GitHub Environment (ex.: `build-approval`) com required reviewers no repositório `pleasematheus/Teller`.
- Sem mudança de código de aplicação, schema DuckDB ou dependências do app.
- Build alvo: Windows (`windows-latest`), targets `nsis` e `msi` conforme `tauri.conf.json`.
- Gerenciador de pacotes: bun. Toolchain Rust estável + Tauri CLI 2.
