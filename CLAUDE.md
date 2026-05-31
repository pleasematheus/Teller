# Teller: Diretrizes do Projeto

## Approach
- Leia arquivos antes de escrever. Não releia sem que tenham mudado.
- Raciocínio minucioso, output conciso.
- Pule arquivos acima de 100KB salvo se necessário.
- Sem aberturas bajuladoras nem fluff de fechamento.
- Sem emojis ou em-dashes.
- Não adivinhe APIs, versões, flags, SHAs ou nomes de pacotes. Verifique lendo código ou docs antes de afirmar.

## Proibido
- Nunca reescreva um arquivo inteiro para alterar uma função; edite apenas o trecho necessário.
- Nunca adicione dependências sem perguntar primeiro.
- Nunca crie testes sem antes ler o arquivo a ser testado.
- Nunca leia o catálogo completo de RTK (docs/rtk.md) sem necessidade explícita.

## Programação Funcional
- Prefira funções puras: mesma entrada, mesma saída, sem side effects ocultos.
- Dados imutáveis: nunca mute objetos ou arrays; use spread, map, filter, reduce.
- Evite classes; use funções e composição.
- Componentes React são funções; side effects só em hooks explícitos (useEffect, useCallback).
- Extraia lógica de negócio de componentes para funções puras testáveis em src/lib/.
- Em Rust, prefira funções que retornam Result<T, E> a mutações implícitas; evite estado mutável compartilhado.

## Testes
- Todo utilitário em src/lib/ tem um *.test.ts correspondente (Vitest).
- Todo Tauri command Rust tem ao menos um teste unitário no mesmo arquivo (#[cfg(test)]).
- Teste comportamento, não implementação.
- Use os blocos #### Scenario: dos specs OpenSpec como referência de casos de teste.
- Testes manuais devem ser incluídos como tasks e delegados a testers humanos, não automatizados.

## Stack
- Frontend: React 19, TypeScript, Tailwind 4, shadcn/ui, Vitest
- Backend: Tauri 2, Rust, DuckDB (duckdb = "1.4.4" bundled)
- Testes: `cargo test` (Rust), `bun run test` (frontend)

## RTK: Comandos Otimizados em Tokens
Sempre prefixe comandos com `rtk`, inclusive em cadeias com `&&`. É sempre seguro: se não houver filtro dedicado, o comando passa through sem alteração.
Exemplos: `rtk cargo build`, `rtk cargo test`, `rtk vitest`, `rtk tsc`, `rtk lint`, `rtk git diff`, `rtk git commit`, `rtk docker logs`.
Estatísticas de economia: `rtk gain`. Catálogo completo de comandos: ver docs/rtk.md.