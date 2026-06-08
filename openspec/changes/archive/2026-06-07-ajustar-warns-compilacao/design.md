## Context

`cargo build` emite 2 warnings e `tsc --noEmit` acusa 4 erros. Nenhum quebra o runtime hoje (o Vite usa transpilação isolada que ignora erros de tipo de testes e algumas checagens), mas o ruído mascara regressões. Esta mudança é puramente higiene de compilação, sem alterar comportamento.

Diagnóstico exato:

Rust (`cargo build` em `src-tauri/`):
- `auth.rs:23`: `pub fn lookup_user` retorna `UserRow`, que é `pub(crate)`. Warning `type 'UserRow' is more private than the item 'lookup_user'`.
- `financials.rs:211`: `fn insert_financial_entry` definida a nível de módulo, fora de `#[cfg(test)]`, nunca chamada em código de produção. Warning `function 'insert_financial_entry' is never used`. (`reports.rs:233` tem cópia própria dentro de `#[cfg(test)]`.)

Frontend (`tsc --noEmit`):
- `EntradaForm.tsx:9`: const `brl` declarada e nunca lida (TS6133).
- `calendar.tsx:89`: chave `table` não existe em `Partial<ClassNames>` do react-day-picker (TS2353).
- `spinner.tsx:7`: `strokeWidth` herdado de `React.ComponentProps<"svg">` é `string | number`, mas `HugeiconsIcon` espera `number` (TS2322).
- `buildPurchaseItemPayload.test.ts:71`: objeto usa `boxQty`, campo ausente em `ItemBuildInput` (TS2353).

## Goals / Non-Goals

**Goals:**
- `cargo build` sem warnings.
- `tsc --noEmit` sem erros.
- Nenhuma mudança de comportamento observável; testes permanecem verdes.

**Non-Goals:**
- Refatorar arquitetura de módulos ou consolidar os dois `insert_financial_entry`.
- Habilitar `tsc` no pipeline de build do Vite (decisão separada).
- Resolver warnings de lint do frontend não acusados por `tsc` (ESLint).

## Decisions

**1. `lookup_user`: restringir função em vez de expor `UserRow`.**
`UserRow` contém `password_hash`, não deve virar API pública. Trocar `pub fn lookup_user` por `pub(crate) fn lookup_user`. Alternativa rejeitada: tornar `UserRow` `pub` — vazaria hash de senha no contrato público do crate.
Verificação necessária: confirmar que nenhum chamador fora do crate usa `lookup_user` (uso atual é interno + testes).

**2. `insert_financial_entry`: mover para dentro de `#[cfg(test)] mod tests`.**
A função só faz sentido como helper de teste (faz `.unwrap()`, sem tratamento de erro). Mover o corpo para dentro do módulo de testes de `financials.rs`. Alternativa rejeitada: anotar `#[allow(dead_code)]` — esconde o problema em vez de resolver. Se nenhum teste do módulo a usar, remover de vez.
Verificação necessária: checar se os testes em `financials.rs` (a partir da linha 234) chamam a função antes de mover/remover.

**3. `brl` não usado: remover a declaração.**
Linha 9 de `EntradaForm.tsx` é morta. Remoção direta.

**4. `calendar.tsx` chave `table`: remover a chave.**
A versão do react-day-picker em uso não aceita `table` em `classNames` (a estrutura usa `month_grid`/`weeks`). Remover a linha `table: "w-full border-collapse"`. Verificar visualmente que o layout do calendário não regride.

**5. `spinner.tsx` `strokeWidth`: fixar o valor e não repassar do spread.**
O componente já passa `strokeWidth={2}` literal. O conflito vem de `{...props}` carregando `strokeWidth?: string | number` do tipo SVG. Solução: tipar props como `Omit<React.ComponentProps<"svg">, "strokeWidth">` para que o literal `2` (number) prevaleça. Alternativa rejeitada: cast `Number(props.strokeWidth)` — mais frágil.

**6. `buildPurchaseItemPayload.test.ts` `boxQty`: alinhar ao tipo atual.**
Ler `ItemBuildInput` e substituir `boxQty` pelo campo correto atual (provável renomeação). Ajustar a asserção do teste conforme o nome vigente, preservando a intenção do cenário.

## Risks / Trade-offs

- [Remover `table` quebra layout do calendário] → Validar visualmente o componente Calendar após a mudança (tarefa de teste manual).
- [Mover/remover `insert_financial_entry` quebra teste] → Conferir chamadas antes; rodar `cargo test` do módulo.
- [`pub(crate)` em `lookup_user` quebra consumidor externo] → Grep por usos fora do crate antes; confirmar que só comandos internos e testes usam.
- [Mudar tipo de props do Spinner afeta call-sites] → `Omit` apenas remove `strokeWidth` da assinatura externa; nenhum call-site passa `strokeWidth` hoje (a verificar com grep).

## Migration Plan

Sem migração de dados ou schema. Aplicar edições, rodar `cargo build`, `cargo test`, `tsc --noEmit`, `bun run test`. Rollback trivial via reversão das edições caso algum teste falhe.

## Open Questions

- Adicionar `tsc --noEmit` e `cargo build -D warnings` ao CI como portão permanente? Fora de escopo aqui, mas recomendado em mudança futura.
