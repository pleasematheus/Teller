## 1. Estado de modalidade único

- [x] 1.1 Em `SaleCloseForm.tsx`, remover o estado `itemModes` e a função `setItemMode`
- [x] 1.2 Remover o estado `globalMode`
- [x] 1.3 Adicionar estado único `const [mode, setMode] = useState<GlobalMode>("%")`

## 2. Ligar leituras ao estado único

- [x] 2.1 Em `updateItemDiscount`, trocar `itemModes[idx] ?? "%"` por `mode`
- [x] 2.2 No render de cada item, trocar `itemMode` (`itemModes[idx] ?? "%"`) por `mode` no rótulo, no cálculo de `inputValue`, no `max` do input e no placeholder
- [x] 2.3 Em `handleGlobalDiscountChange`, trocar `globalMode` por `mode`

## 3. Seletor único na UI

- [x] 3.1 Remover o par de botões `R$ / %` do cabeçalho de cada item; manter só o rótulo `Desconto ({mode})` e o indicador de máximo
- [x] 3.2 Atualizar o par de botões do bloco "Desconto global" para chamar `setMode(m)` e limpar `globalDiscount`, refletindo `mode`
- [x] 3.3 Garantir que alternar o seletor reexibe os campos dos itens convertidos (frações preservadas em `cartItems`) e limpa o campo global

## 4. Verificação

- [x] 4.1 Rodar `bun run test` (subagente) e confirmar que nenhum teste de `pricing` quebrou
- [x] 4.2 Rodar typecheck/build do frontend e confirmar ausência de referências a `itemModes`/`globalMode`
- [x] 4.3 [MANUAL - testador humano] Abrir fechamento de venda, aplicar descontos por item e global, alternar o seletor único entre `%` e `R$` e confirmar que todos os campos mudam juntos com as frações preservadas
