## Context

`SaleCloseForm.tsx` hoje mantém três fontes de modalidade de desconto:

- `itemModes: Record<number, GlobalMode>` — modalidade por item (default `%` via `itemModes[idx] ?? "%"`).
- `globalMode: GlobalMode` — modalidade do desconto global (default `R$`).
- Cada um renderiza seu próprio par de botões `R$ / %`.

O desconto efetivo de cada item já é armazenado como fração (`CartItem.discount`, 0-1) independentemente da modalidade. A modalidade é puramente estado de interface: controla como o valor é exibido e como a entrada digitada é convertida em fração.

## Goals / Non-Goals

**Goals:**
- Um único seletor de modalidade compartilhado controla a exibição de todos os campos de desconto (itens + global).
- Alternar a modalidade reexibe todos os valores na nova unidade, preservando as frações já aplicadas.
- Nenhuma mudança no contrato com o backend (`close_sale` recebe frações 0-1).

**Non-Goals:**
- Alterar a matemática de cálculo de desconto, piso de margem ou rateio global.
- Alterar backend, schema ou tipos enviados ao Tauri.
- Mudar o default inicial da modalidade (continua `%`).

## Decisions

**Decisão: estado único `mode: GlobalMode` substitui `itemModes` e `globalMode`.**
- Remover `itemModes` e `setItemMode`; remover `globalMode`.
- Introduzir `const [mode, setMode] = useState<GlobalMode>("%")`.
- Todos os locais que liam `itemModes[idx] ?? "%"` ou `globalMode` passam a ler `mode`.
- Alternativa considerada: manter modos independentes e adicionar um botão "aplicar a todos". Rejeitada: mantém a complexidade de N estados e não atende ao pedido de linkar os seletores; o operador ainda poderia divergir os modos.

**Decisão: um único par de botões `R$ / %` na tela.**
- Os botões por item são removidos; o cabeçalho de cada item passa a exibir só o rótulo `Desconto (R$|%)` derivado de `mode`.
- O par de botões compartilhado fica junto ao seletor de desconto global (controle de modalidade da tela), e ao alternar atualiza item e global simultaneamente por já lerem `mode`.
- Alternativa: colocar o seletor único no topo da lista de itens. Equivalente funcionalmente; manter junto ao global reduz reorganização de layout.

**Decisão: alternar modalidade preserva frações dos itens e limpa o campo global.**
- As frações em `CartItem.discount` não são tocadas ao chamar `setMode`, então os campos por item se reexibem convertidos (comportamento atual da função `inputValue`).
- O campo de entrada do desconto global (`globalDiscount`, string transitória) é limpo ao alternar, preservando o comportamento atual de `setGlobalMode(m); setGlobalDiscount("")`.

## Risks / Trade-offs

- [Operador perde a capacidade de ter itens em modalidades diferentes] → É exatamente o objetivo pedido; a fração final por item continua ajustável individualmente, só a unidade de exibição é unificada.
- [Regressão visual no cabeçalho do item ao remover os botões] → Coberto por teste manual de fechamento; lógica de conversão fica isolada na função `inputValue`/`updateItemDiscount`.
