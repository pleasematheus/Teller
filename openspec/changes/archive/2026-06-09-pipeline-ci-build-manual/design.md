## Context

Teller é um app Tauri 2 (React 19 + Rust + DuckDB) que produz instaladores Windows (`nsis`, `msi`). Não há CI/CD. Os testes rodam via `bun run test` (Vitest) e `cargo test`; o build do bundle via `bun run tauri build`. O repositório remoto é `pleasematheus/Teller` no GitHub, então GitHub Actions é a plataforma natural.

O requisito central: verificação contínua deve ser automática, mas o build do instalador (operação cara, ~minutos, e que gera artefato distribuível) só pode ocorrer com autorização explícita do mantenedor.

## Goals / Non-Goals

**Goals:**
- CI automático em push e PR: testes, tipos e lint para frontend e Rust.
- Build do instalador Windows gated: nunca dispara sozinho; exige ação ou aprovação humana.
- Build só prossegue se o CI estiver verde.
- Artefatos (instaladores) disponíveis para download após build autorizado.

**Non-Goals:**
- Assinatura de código (code signing) dos instaladores.
- Auto-update / publicação automática em canal de release.
- Builds para macOS/Linux (apenas Windows).
- Cache otimizado avançado além do básico (pode vir depois).

## Decisions

### D1: Dois workflows separados (`ci.yml` + `build.yml`)
Separar verificação (rápida, frequente) do build (caro, sob demanda) deixa o gate explícito e os logs limpos.
- `ci.yml`: gatilhos `push` e `pull_request`. Roda em `windows-latest` (DuckDB bundled + targets Windows tornam a paridade de SO desejável; aceitável usar `ubuntu-latest` só para os testes JS, mas Rust+duckdb compila melhor no alvo final — escolhemos `windows-latest` para fidelidade).
- `build.yml`: gatilho `workflow_dispatch` (disparo manual) apenas.

Alternativa considerada: um único workflow com job de build condicionado a `if: github.event_name == 'workflow_dispatch'`. Rejeitada por misturar responsabilidades e dificultar a leitura do gate.

### D2: Gate manual em duas camadas
1. **`workflow_dispatch`**: o build só existe como disparo manual a partir da aba Actions (ou `gh workflow run build.yml`). Não há gatilho automático.
2. **GitHub Environment `build-approval`**: o job de build referencia `environment: build-approval`, configurada com *required reviewers*. Mesmo disparado, o job fica pendente até aprovação humana na UI.

A camada 2 cobre o caso de disparo acidental e permite revisores distintos do disparador. Alternativa (só `workflow_dispatch`) é mais simples mas não exige segunda confirmação; mantemos ambas, com a Environment configurável/opcional.

### D3: Build depende de CI verde
`build.yml` primeiro executa um job de verificação (reusa os mesmos passos do CI, idealmente via `workflow_call` reutilizável) e o job de bundle declara `needs: [verify]`. Assim build autorizado sobre código quebrado falha cedo.

Alternativa: confiar no status do último CI da branch. Rejeitada: estado pode estar defasado em relação ao commit disparado.

### D4: Toolchain e cache
- `oven-sh/setup-bun` para bun; `bun install --frozen-lockfile`.
- Rust via `dtolnay/rust-toolchain@stable` + `Swatinem/rust-cache` para acelerar `cargo`.
- Componentes `rustfmt`, `clippy` para os checks.
- Passos de verificação: `tsc --noEmit` (via `bun run build` ou check dedicado), `bun run test`, `cargo fmt --check`, `cargo clippy -- -D warnings`, `cargo test` em `src-tauri`.

### D5: Artefatos
Após build, `actions/upload-artifact` publica os instaladores de `src-tauri/target/release/bundle/{nsis,msi}/`. Release no GitHub fica como passo opcional, controlado por input booleano do `workflow_dispatch` (ex.: `create_release`), default `false`.

## Risks / Trade-offs

- [Build no Windows é lento sem cache] → `Swatinem/rust-cache` + `--frozen-lockfile`; aceitar primeiros runs lentos.
- [Environment `build-approval` exige config manual no GitHub (não versionável no repo)] → documentar nos tasks como passo manual; workflow funciona mesmo sem ela (degrada para apenas `workflow_dispatch`).
- [Duplicação de passos entre `ci.yml` e verificação do `build.yml`] → extrair workflow reutilizável (`workflow_call`) se a duplicação incomodar; começar simples.
- [`cargo clippy -- -D warnings` pode quebrar build por warnings preexistentes] → rodar localmente antes; se houver muitos warnings, relaxar para não-bloqueante inicialmente.
- [Segredos para release/signing] → fora de escopo; sem segredos necessários para build não assinado.

## Migration Plan

1. Adicionar `.github/workflows/ci.yml`; validar em PR de teste que roda e passa.
2. Adicionar `.github/workflows/build.yml` com `workflow_dispatch`.
3. Criar Environment `build-approval` com required reviewers no GitHub (manual).
4. Disparar build manual de teste e confirmar gate de aprovação + artefatos.

Rollback: remover os arquivos de workflow; nenhum impacto em código de app.

## Open Questions

- Manter `windows-latest` para o CI inteiro ou separar testes JS em `ubuntu-latest` para velocidade? (Default: tudo em `windows-latest`.)
- `cargo clippy` bloqueante (`-D warnings`) desde o início ou warning-only? (Default: bloqueante; reavaliar se falhar.)
