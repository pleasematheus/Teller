## 1. Função pura de ordenação

- [x] 1.1 Adicionar `sortBy<T>(items, value, dir)` e tipo `SortDir` em `src/lib/listSearch.ts`: acessor string/number, números numéricos, texto via localeCompare pt-BR, nulos sempre ao fim, novo array imutável
- [x] 1.2 Escrever testes em `src/lib/listSearch.test.ts`: asc/desc numérico, asc/desc textual com acento, nulos ao fim em ambas direções, estabilidade em empates, array vazio

## 2. Integração na tela de estoque

- [x] 2.1 Adicionar tipos `SortColumn`/`SortState` e estado `sort` em `StockBalance.tsx`
- [x] 2.2 Definir mapa de acessores `SORT_VALUE` por coluna (name, unit, quantity, capacity com max e null)
- [x] 2.3 Transformar cabeçalhos Produto, Unidade, Quantidade e Capacidade em botões clicáveis com ciclo asc -> desc -> off e troca de coluna iniciando em asc
- [x] 2.4 Adicionar indicador visual (seta) no cabeçalho da coluna ativa refletindo a direção
- [x] 2.5 Aplicar `sortBy` ao final do pipeline (`byType -> rankByName -> sort`) quando `sort` definido

## 3. Validação

- [x] 3.1 Rodar `bun run test` e confirmar suíte verde (delegar a subagente)
- [x] 3.2 Rodar type check e confirmar zero erros novos
- [x] 3.3 [MANUAL] Testador humano: verificar ciclo por coluna, troca entre colunas, precedência sobre busca, capacidade pela maior estimativa com vazios ao fim, combinação com filtro de tipo
