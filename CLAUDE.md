# Teller — Diretrizes do Projeto

## Programação Funcional

- Prefira funções puras: mesma entrada, mesma saída, sem side effects ocultos
- Dados imutáveis: nunca mute objetos ou arrays diretamente; use spread, `map`, `filter`, `reduce`
- Evite classes; use funções e composição
- Componentes React devem ser funções — sem lógica com side effects fora de hooks explícitos (`useEffect`, `useCallback`)
- Extraia lógica de negócio de componentes para funções puras testáveis em arquivos separados (`src/lib/`)
- No Rust, prefira funções que retornam `Result<T, E>` a mutações implícitas; evite estado mutável compartilhado

## Testes

- Todo utilitário em `src/lib/` deve ter um arquivo de teste correspondente (`*.test.ts`) com Vitest
- Todo Tauri command Rust deve ter ao menos um teste unitário no mesmo arquivo (`#[cfg(test)]`)
- Teste comportamento, não implementação: teste o que a função faz, não como ela faz
- Cada cenário dos specs OpenSpec (`#### Scenario:`) é um teste potencial — use-os como referência

## Stack

- Frontend: React 19, TypeScript, Tailwind 4, shadcn/ui, Vitest
- Backend: Tauri 2, Rust, DuckDB (`duckdb = "1.4.4" bundled`)
- Testes Rust: `cargo test`; Testes frontend: `bun run test` (Vitest)
