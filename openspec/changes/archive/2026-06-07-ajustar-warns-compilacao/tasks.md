## 1. Warnings Rust

- [x] 1.1 Grep por usos de `lookup_user` fora do crate `Teller`; confirmar que só comandos internos e testes a usam.
- [x] 1.2 Em `src-tauri/src/auth.rs:23`, trocar `pub fn lookup_user` por `pub(crate) fn lookup_user`.
- [x] 1.3 Em `src-tauri/src/financials.rs`, verificar se algum teste (a partir da linha 234) chama `insert_financial_entry`.
- [x] 1.4 Se usado por testes, mover `insert_financial_entry` (linha 211) para dentro de `#[cfg(test)] mod tests`; se não usado, remover a função.
- [x] 1.5 Rodar `cargo build` em `src-tauri/` e confirmar zero warnings.

## 2. Erros TypeScript

- [x] 2.1 Remover a const `brl` não usada em `src/components/financeiro/EntradaForm.tsx:9`.
- [x] 2.2 Remover a chave `table: "w-full border-collapse"` de `classNames` em `src/components/ui/calendar.tsx:89`.
- [x] 2.3 Em `src/components/ui/spinner.tsx`, tipar props como `Omit<React.ComponentProps<"svg">, "strokeWidth">` para resolver o conflito de `strokeWidth`.
- [x] 2.4 Ler `ItemBuildInput` e substituir o campo obsoleto `boxQty` pelo campo válido atual em `src/lib/buildPurchaseItemPayload.test.ts:71`, preservando a intenção do cenário.
- [x] 2.5 Rodar `bun run tsc --noEmit` e confirmar zero erros.

## 3. Verificação

- [x] 3.1 Rodar `cargo test` (subagente) e confirmar todos os testes verdes, incluindo os três testes de `lookup_user`.
- [x] 3.2 Rodar `bun run test` (subagente) e confirmar todos os testes verdes, incluindo a suíte `buildPurchaseItemPayload`.
- [x] 3.3 [MANUAL - testador humano] Abrir o componente Calendar na UI e confirmar que o layout não regrediu após remover a chave `table`.
- [x] 3.4 [MANUAL - testador humano] Confirmar visualmente que o Spinner ainda renderiza e anima corretamente.
