## Context

`PurchaseForm` é a tela de entrada de compra multi-item. Cada linha tem um campo "Qtd" (`item.quantity`, string). A unidade do item vem do produto selecionado (`productUnit`) ou do novo produto (`newProductUnit`), e o flag inteiro é resolvido por `isUnitInteger(units, unitName)`.

Estado atual:
- O input usa `step={itemInteger ? "1" : "any"}` e `min={itemInteger ? 1 : 0.0001}`. Em navegadores, `step` não impede digitação de decimais; só afeta os botões de incremento e a validação nativa do form, que não é acionada aqui.
- Feedback inline (`qtyInvalid` -> "Apenas inteiros") já existe.
- O submit chama `validateQuantity` e bloqueia salvar com mensagem agregada.

A tela de venda (`SaleForm`) já resolve isso normalizando na digitação: `updateQuantity` faz `integer ? Math.max(1, Math.round(value)) : Math.max(0.001, value)`. A inconsistência entre as duas telas é a origem do bug reportado.

Há também o caminho de conversão "Unidades por caixa" (`unit_qty`): a quantidade que entra em estoque é `quantity * unit_qty`. Se a unidade do produto é inteira mas `unit_qty` é fracionado (ou o produto é fracionado nessa linha), o estoque resultante pode ser fracionado, o que viola a mesma regra de integridade.

## Goals / Non-Goals

**Goals:**
- Impedir que o operador registre quantidade fracionada para itens de compra cuja unidade é inteira.
- Normalizar a quantidade durante a digitação, consistente com `SaleForm`.
- Garantir que a quantidade de estoque resultante da conversão caixa -> unidade respeite a integridade quando a unidade do produto for inteira.
- Reusar lógica pura entre venda e compra para evitar divergência futura.

**Non-Goals:**
- Não alterar validação de backend nem o schema DuckDB (a regra é de UI; o backend já rejeita `quantity <= 0`).
- Não mudar o comportamento de unidades fracionadas (kg etc.).
- Não unificar toda a lógica de quantidade de `SaleForm` e `PurchaseForm` num único componente.

## Decisions

**1. Normalizar na digitação, não só no submit.**
Adicionar normalização no `onChange` do campo "Qtd" do `PurchaseForm`, espelhando `SaleForm`. Para unidade inteira, descartar a parte decimal ao confirmar o valor. Alternativa considerada: bloquear teclas de ponto/vírgula no `onKeyDown`. Rejeitada por ser frágil (colar, IME, locale do separador decimal) e não cobrir o caminho de conversão.

Detalhe de UX: arredondar de forma agressiva a cada tecla atrapalha a digitação. Decisão: manter a string crua enquanto o usuário digita e exibir o erro inline já existente (`qtyInvalid`), mas truncar para inteiro no `onBlur` e bloquear no submit. Isso preserva a edição fluida e ainda impede o valor inválido de persistir. Como `SaleForm` arredonda no `onChange`, extrair a regra para função pura compartilhada e aplicá-la no ponto adequado de cada tela.

**2. Função pura compartilhada em `src/lib/unitQuantity.ts`.**
Adicionar `normalizeQuantity(value: number, integer: boolean): number` (inteiro -> `Math.trunc`/`Math.round` com piso mínimo; fracionado -> valor com piso mínimo). Mantém `validateQuantity` como guarda de submit. Justificativa: lógica de negócio pura e testável fora do componente, conforme as diretrizes do projeto.

**3. Validar a quantidade de estoque da conversão.**
No submit, além de `validateQuantity(quantity)`, quando `showConversion` e a unidade do produto for inteira, validar também `validateQuantity(stockQty(item), true)`. Se fracionado, bloquear com mensagem específica. Alternativa: arredondar o estoque automaticamente. Rejeitada por mascarar erro de cadastro de `unit_qty`; melhor exigir correção explícita.

## Risks / Trade-offs

- [Truncar no `onBlur` pode surpreender quem digitou "1.9" esperando 2] -> usar `Math.round` para inteiro (não trunc), alinhado a `SaleForm`, e manter o erro inline visível enquanto o valor é fracionado.
- [Unidade ausente na lista assume `integer = true` (default de `isUnitInteger`)] -> comportamento já existente e coerente com a coluna; nenhum risco novo.
- [Divergência futura entre venda e compra] -> mitigada pela função pura compartilhada com testes.
