## Context

Na tela de fechamento (`SaleCloseForm.tsx`), o desconto por item é editado por um único `Input` em porcentagem: `updateItemDiscount` faz `value / 100` e guarda como fração em `CartItem.discount` (0-1). O backend `do_close_sale_work` recebe `discount` como fração e calcula `discounted_price = price × (1 − discount)`. O desconto global já tem um seletor R$/% (`globalMode`) que demonstra o padrão de UI desejado. A peça que falta é a alternância de modalidade no nível do item.

## Goals / Non-Goals

**Goals:**
- Permitir que o operador informe o desconto de cada item em valor absoluto (R$) ou porcentagem (%).
- Reusar o padrão visual do seletor R$/% já existente no desconto global.
- Manter o backend e o schema inalterados: o que trafega para `close_sale` continua sendo a fração 0-1.

**Non-Goals:**
- Persistir a modalidade escolhida (R$ ou %) no banco. Modalidade é estado de UI; o que se persiste é a fração resultante.
- Alterar o desconto global ou a lógica de rateio.
- Alterar o cálculo do piso de margem (`computeMaxDiscount`).

## Decisions

### Modalidade por item como estado local indexado
A modalidade vive no componente como `itemModes: Record<number, GlobalMode>` (ou array paralelo a `cartItems`), default `"%"` para preservar o comportamento atual. Alternativa considerada: adicionar `mode` a `CartItem`. Rejeitada porque `CartItem` espelha dados de domínio (preço, custo, desconto) e a modalidade é puramente de apresentação; manter separado evita poluir o tipo de dados.

### Conversão valor→fração como função pura em pricing.ts
Adicionar `itemDiscountFraction(value, lineTotal)` que retorna `clamp(value / lineTotal, 0, 1)` onde `lineTotal = price × quantity`. Isso espelha a regra já usada em `applyGlobalDiscount` (rateio sobre `price × qty`) e fica testável isoladamente em `pricing.test.ts`. No modo %, a conversão permanece `value / 100`.

Decisão de base do valor: o desconto em R$ é interpretado sobre o total da linha (`price × quantity`), coerente com o desconto global e com o subtotal exibido. Alternativa: aplicar sobre o preço unitário. Rejeitada por inconsistência com o desconto global, que rateia sobre o valor total da linha.

### Indicador de máximo sensível à modalidade
`computeMaxDiscount` já devolve `maxDiscountValue` e `maxDiscountPct`. No modo R$, o hint destaca o limite em R$; no modo %, em %. Ambos continuam exibidos no texto `· máx. R$ X (Y%)`, então a mudança é apenas de ênfase/placeholder do input, não de cálculo.

## Risks / Trade-offs

- [Arredondamento ao alternar modalidade] Converter % → R$ → % pode introduzir centavos de diferença. → Mitigação: a fonte de verdade é sempre a fração armazenada; ao trocar de modalidade apenas reexibimos o valor derivado da fração, sem regravar, evitando drift acumulado.
- [Linha com `quantity` ou `price` zero] Divisão por zero na conversão R$→fração. → Mitigação: a função pura retorna 0 quando `lineTotal <= 0`, como `applyGlobalDiscount` já faz.
- [Interação com desconto global] Aplicar desconto global sobrescreve `discount` de todos os itens (comportamento atual). A modalidade por item continua sendo só de exibição, então não conflita; após o global, o item exibe o desconto resultante na modalidade ativa.
