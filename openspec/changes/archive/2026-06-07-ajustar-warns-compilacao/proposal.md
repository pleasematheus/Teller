## Why

A compilação atual emite ruído que mascara problemas reais: o backend Rust gera 2 warnings (`cargo build`) e o `tsc --noEmit` do frontend acusa 4 erros de tipo/variável não usada. Build limpo (zero warnings, zero erros) facilita detectar regressões e mantém a barra de qualidade.

## What Changes

- Corrigir warning Rust de visibilidade: `lookup_user` é `pub` mas retorna `UserRow` que é `pub(crate)` (`auth.rs:23`).
- Corrigir warning Rust de código morto: `insert_financial_entry` nunca usado (`financials.rs:211`).
- Remover variável não usada `brl` em `EntradaForm.tsx:9` (TS6133).
- Remover chave inválida `table` do `classNames` em `calendar.tsx:89` (TS2353, incompatível com `Partial<ClassNames>` do react-day-picker).
- Corrigir conflito de tipo de `strokeWidth` no spread de props SVG em `spinner.tsx:7` (TS2322).
- Atualizar `buildPurchaseItemPayload.test.ts:71` que usa campo obsoleto `boxQty` fora de `ItemBuildInput` (TS2353).

Sem mudança de comportamento em runtime. Apenas higiene de compilação.

## Capabilities

### New Capabilities
- `compilacao-limpa`: Estabelece a invariante de que `cargo build` e `tsc --noEmit` devem completar sem warnings nem erros, como portão de qualidade do projeto.

### Modified Capabilities
<!-- Nenhuma. Nenhum comportamento de runtime ou requisito de spec existente muda. -->

## Impact

- Código Rust: `src-tauri/src/auth.rs`, `src-tauri/src/financials.rs`.
- Código frontend: `src/components/financeiro/EntradaForm.tsx`, `src/components/ui/calendar.tsx`, `src/components/ui/spinner.tsx`, `src/lib/buildPurchaseItemPayload.test.ts`.
- APIs/dependências: nenhuma. `lookup_user` continua acessível dentro do crate; nenhum consumidor externo afetado.
- Testes: `cargo test` e `bun run test` devem continuar verdes; o teste de payload de compra passa a usar campo válido.
