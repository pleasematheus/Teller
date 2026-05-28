## ADDED Requirements

### Requirement: Separação em módulos por domínio
O backend Rust SHALL ser organizado em módulos separados por domínio de negócio. Cada domínio MUST ter seu próprio arquivo `.rs` em `src-tauri/src/`. O arquivo `lib.rs` MUST conter apenas declarações `mod`, a função `run()` e o `generate_handler![]`.

#### Scenario: Módulos existem como arquivos separados
- **WHEN** o projeto é compilado
- **THEN** existem os arquivos `db.rs`, `auth.rs`, `products.rs`, `stock.rs`, `sales.rs`, `settings.rs` e `financials.rs` em `src-tauri/src/`

#### Scenario: lib.rs não contém lógica de negócio
- **WHEN** se lê o arquivo `lib.rs`
- **THEN** ele contém apenas declarações `mod`, a função `run()` e o macro `generate_handler![]`, sem structs ou funções de domínio

### Requirement: Testes colocalizados com o código
Cada módulo SHALL ter seu próprio bloco `#[cfg(test)] mod tests` com os testes pertinentes ao seu domínio. Testes MUST residir no mesmo arquivo que o código que testam.

#### Scenario: Testes de um módulo estão no mesmo arquivo
- **WHEN** se busca os testes de um domínio específico (ex: produtos)
- **THEN** eles estão em `products.rs`, dentro de `#[cfg(test)] mod tests`

#### Scenario: Helpers de teste são pub(crate)
- **WHEN** um helper de teste é necessário em múltiplos módulos
- **THEN** ele está definido como `pub(crate)` no módulo de origem (`test_db` em `db.rs`, `insert_user` em `auth.rs`)

### Requirement: Ausência de dependências circulares entre módulos
Os módulos SHALL seguir uma hierarquia de dependência sem ciclos. `db` MUST ser o único módulo sem dependências internas. Módulos de domínio MUST importar apenas de `db` ou de módulos de nível inferior.

#### Scenario: Compilação sem dependências circulares
- **WHEN** `cargo build` é executado
- **THEN** compila sem erros de dependência circular

### Requirement: Paridade de comportamento após refactor
Todos os comandos Tauri expostos ao frontend SHALL funcionar de forma idêntica antes e depois do refactor. Nenhuma lógica de negócio MUST ser alterada durante a migração.

#### Scenario: Testes passam antes e depois de cada extração
- **WHEN** um módulo é extraído de `lib.rs`
- **THEN** `cargo test` passa com o mesmo número de testes e sem falhas

#### Scenario: Comandos Tauri permanecem acessíveis
- **WHEN** o frontend chama um comando Tauri após o refactor
- **THEN** o comando responde de forma idêntica à versão anterior
