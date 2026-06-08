## ADDED Requirements

### Requirement: Verificação contínua automática
A pipeline SHALL executar automaticamente, em todo `push` e `pull_request`, a verificação completa do código: testes do frontend (`bun run test`), testes do backend (`cargo test`), checagem de tipos TypeScript e checagens de lint/formatação Rust (`cargo fmt --check`, `cargo clippy`).

#### Scenario: Push aciona verificação
- **WHEN** um commit é enviado para qualquer branch monitorada
- **THEN** o workflow de CI inicia automaticamente e executa testes, checagem de tipos e lint

#### Scenario: Pull request aciona verificação
- **WHEN** um pull request é aberto ou atualizado
- **THEN** o workflow de CI executa e reporta status de sucesso ou falha no PR

#### Scenario: Falha de teste reprova o CI
- **WHEN** qualquer teste, checagem de tipo ou lint falha
- **THEN** o workflow de CI termina com status de falha

### Requirement: Build do instalador não dispara automaticamente
A pipeline SHALL NOT iniciar o build do instalador Windows em resposta a `push` ou `pull_request`. O build SHALL ser iniciado apenas por disparo manual (`workflow_dispatch`).

#### Scenario: Push não dispara build
- **WHEN** um commit é enviado para uma branch
- **THEN** nenhum job de build do instalador é iniciado

#### Scenario: Disparo manual inicia o build
- **WHEN** o mantenedor dispara o workflow de build manualmente pela aba Actions ou via `gh workflow run`
- **THEN** o workflow de build é iniciado

### Requirement: Gate de aprovação humana para o build
O job de build do instalador SHALL referenciar uma GitHub Environment protegida por aprovação manual (required reviewers), de forma que, mesmo após disparo manual, o build permaneça pendente até que um revisor autorizado aprove.

#### Scenario: Build aguarda aprovação
- **WHEN** o workflow de build é disparado manualmente
- **THEN** o job de bundle fica em estado pendente aguardando aprovação na Environment de build

#### Scenario: Aprovação libera o build
- **WHEN** um revisor autorizado aprova a execução pendente
- **THEN** o job de bundle prossegue e gera o instalador

#### Scenario: Rejeição cancela o build
- **WHEN** um revisor autorizado rejeita a execução pendente
- **THEN** o job de bundle não executa e o workflow termina sem gerar artefatos

### Requirement: Build exige CI verde
O job de build do instalador SHALL executar os mesmos checks de verificação (testes, tipos, lint) e SHALL prosseguir para o bundle apenas se essa verificação for bem-sucedida.

#### Scenario: Verificação falha bloqueia bundle
- **WHEN** a verificação dentro do workflow de build falha
- **THEN** o job de bundle não executa e nenhum instalador é gerado

#### Scenario: Verificação verde libera bundle
- **WHEN** a verificação dentro do workflow de build passa e a aprovação foi concedida
- **THEN** o job de bundle gera os instaladores

### Requirement: Geração e disponibilização de artefatos
Após um build bem-sucedido, a pipeline SHALL produzir os instaladores Windows nos targets configurados (`nsis` e `msi`) e SHALL disponibilizá-los como artefatos do workflow para download.

#### Scenario: Artefatos disponíveis após build
- **WHEN** o job de bundle conclui com sucesso
- **THEN** os instaladores `.exe` (nsis) e `.msi` são enviados como artefatos do workflow

#### Scenario: Release opcional
- **WHEN** o disparo manual indica criação de release (input habilitado)
- **THEN** a pipeline cria uma GitHub Release anexando os instaladores
