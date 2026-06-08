## ADDED Requirements

### Requirement: Build Rust sem warnings
O backend Rust SHALL compilar via `cargo build` sem emitir nenhum warning.

#### Scenario: cargo build limpo
- **WHEN** `cargo build` é executado em `src-tauri/`
- **THEN** a saída não contém nenhuma linha iniciada por `warning:`
- **AND** o comando termina com código de saída 0

#### Scenario: visibilidade consistente em lookup_user
- **WHEN** `lookup_user` retorna o tipo `UserRow`
- **THEN** a visibilidade da função não é mais ampla que a do tipo retornado
- **AND** nenhum warning `type ... is more private than the item` é emitido

#### Scenario: ausência de código morto em financials
- **WHEN** `cargo build` compila o módulo `financials`
- **THEN** nenhum warning `function ... is never used` é emitido para `insert_financial_entry`

### Requirement: Frontend sem erros de tipo
O frontend SHALL passar em `tsc --noEmit` sem nenhum erro de compilação TypeScript.

#### Scenario: typecheck limpo
- **WHEN** `bun run tsc --noEmit` é executado
- **THEN** nenhum erro `TS` é reportado
- **AND** o comando termina com código de saída 0

#### Scenario: sem variáveis ou imports não usados
- **WHEN** o typecheck analisa `EntradaForm.tsx`
- **THEN** nenhum erro TS6133 (declarado mas nunca usado) é reportado

#### Scenario: classNames do calendar compatível com a API
- **WHEN** o typecheck analisa `calendar.tsx`
- **THEN** o objeto `classNames` contém apenas chaves válidas de `Partial<ClassNames>` do react-day-picker
- **AND** nenhum erro TS2353 é reportado

#### Scenario: tipos de props do Spinner consistentes
- **WHEN** o typecheck analisa `spinner.tsx`
- **THEN** nenhum erro TS2322 de incompatibilidade de tipo em `strokeWidth` é reportado

#### Scenario: teste de payload de compra usa campos válidos
- **WHEN** o typecheck analisa `buildPurchaseItemPayload.test.ts`
- **THEN** o objeto de entrada usa apenas campos definidos em `ItemBuildInput`
- **AND** nenhum erro TS2353 é reportado

### Requirement: Suítes de teste permanecem verdes
A limpeza de compilação SHALL preservar o comportamento existente, mantendo todas as suítes de teste aprovadas.

#### Scenario: testes Rust passam
- **WHEN** `cargo test` é executado
- **THEN** todos os testes passam, incluindo `lookup_user_finds_by_nickname_case_insensitive`, `lookup_user_finds_by_numeric_id` e `lookup_user_returns_none_for_unknown`

#### Scenario: testes frontend passam
- **WHEN** `bun run test` é executado
- **THEN** todos os testes passam, incluindo a suíte `buildPurchaseItemPayload`
