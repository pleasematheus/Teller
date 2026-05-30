## Context

A precificação do Teller usa **markup divisor** ("margem por dentro"), onde CV%, DF%, Investimento% e Margem% são frações do preço de venda:

```
final_price = CMV / (1 − (cv + df + investment_rate + margin))
CMV = cost_price / (1 − expected_loss_rate)
```

Hoje `computePrice` (`src/lib/pricing.ts:3`) recebe `profit_margin` como entrada e devolve `final_price`. As taxas `cv`, `df` e `investment_rate` vêm de `get_pricing_rates`, derivadas do histórico agregado da empresa (despesas, faturamento e taxas de cartão dos últimos 12 meses). Logo, o `final_price` recalculado no formulário muda sempre que esse histórico muda — o preço de etiqueta é instável por construção.

`computePrice` é consumido apenas em `ProductForm.tsx`. `SaleForm.tsx` lê o `final_price` já persistido em `Products` e usa `computeMaxDiscount`; não chama `computePrice`. O schema `Products` já guarda `final_price`, `markup` e `profit_margin`, e os comandos `create_product`/`update_product` já recebem os três.

## Goals / Non-Goals

**Goals:**
- Tornar o preço de venda (`final_price`) a variável de entrada estável, definida pelo operador.
- Derivar `profit_margin` do preço final, como leitura analítica (pode ser negativa).
- Impedir precificação no prejuízo usando o CMV como piso de custo.
- Manter o cálculo como função pura e testável; reaproveitar `computePrice` apenas para sugerir um preço inicial a partir da margem realizada.

**Non-Goals:**
- Alterar a fórmula de markup divisor (a álgebra é a mesma, só se inverte entrada/saída).
- Mudar schema do banco, comandos Tauri ou o fluxo de vendas/desconto.
- Mudar `get_realized_margin` ou `get_pricing_rates`.
- Recalcular preços de produtos já cadastrados em massa (o preço persistido permanece como está até o operador editar).

## Decisions

**1. Inverter a direção do cálculo com uma nova função `computeMarginFromPrice`, mantendo `computePrice`.**
A partir de `final_price = CMV / (1 − (cv + df + ir + margin))`, isola-se a margem:

```
margin = 1 − cv − df − investment_rate − CMV / final_price
markup_percent = final_price / CMV
```

`computeMarginFromPrice(costPrice, lossRate, finalPrice, rates)` retorna um `MarginBreakdown` com `cmv`, `markup_percent`, `profit_margin` (derivada, podendo ser negativa), `min_price`, `break_even_price`, `below_cost` e `feasible`. `computePrice` continua existindo porque a sugestão de preço inicial (decisão 4) precisa do caminho margem→preço. Alternativa descartada: substituir `computePrice` e calcular a sugestão "na mão" no componente — pioraria a testabilidade.

**2. CMV é o piso de custo (`min_price = CMV`); abaixo dele é prejuízo e bloqueia o salvamento.**
Vender abaixo do CMV é perder dinheiro no próprio item (já considerada a perda). `below_cost = final_price < CMV`; nesse caso `feasible = false` e o formulário bloqueia o submit com aviso de prejuízo. Entre o CMV e o `break_even_price`, a `profit_margin` derivada é **negativa** (cobre o custo do bem, mas não a parcela de overhead): o sistema **permite**, mas exibe aviso. Acima do `break_even_price`, a margem é positiva. Essa é a leitura direta de "colocar o preço do item e CMV como custo mínimo para não precificar no prejuízo".

**3. `break_even_price` é o preço de margem zero e referência de saúde do preço.**

```
break_even_price = CMV / (1 − cv − df − investment_rate)   (quando 1 − cv − df − ir > 0)
```

Quando `1 − cv − df − investment_rate <= 0` (as taxas já consomem todo o preço), nenhum preço positivo gera margem ≥ 0: `break_even_price` é indefinido, `feasible = false` para qualquer preço, e o formulário comunica que as taxas consomem o preço. Isso preserva o aviso de inviabilidade da mudança anterior, agora no espaço de preço.

**4. A sugestão automática passa do campo de margem para o campo de preço.**
Hoje o formulário pré-popula o campo de margem com `get_realized_margin`. Como a margem deixa de ser editável, o formulário passa a pré-popular o **preço sugerido** = `computePrice(cost, loss, rates, realizedMargin).final_price`. Sem histórico (`realizedMargin = 0`), a sugestão recai sobre o `break_even_price` (margem zero) e o aviso de "sem histórico" permanece. Na edição, o campo de preço é pré-populado com o `final_price` persistido.

**5. `markup` e `profit_margin` continuam persistidos, como snapshot derivado no momento do salvamento.**
A fonte de verdade no cliente passa a ser `final_price`; `markup = final_price/CMV` e `profit_margin` derivada são recalculados e gravados a cada salvamento, mantendo o contrato dos comandos Tauri e os campos usados em relatórios. Não viram fonte de verdade — são leitura congelada no instante do save.

**6. O markup não é exibido no formulário; apenas custo, CMV e margem derivada.**
O `markup_percent` continua sendo calculado por `computeMarginFromPrice` e persistido (decisão 5), mas não aparece na UI do `ProductForm`. O bloco de leitura mostra somente o CMV (o custo já é um campo de entrada) e a margem de lucro derivada, evitando ruído de uma métrica intermediária que o operador não precisa interpretar.

## Risks / Trade-offs

- **[`profit_margin` persistida vira snapshot e pode "envelhecer" se as taxas mudarem]** → É aceitável: a margem realizada para relatórios vem de `get_realized_margin` (calculada das vendas), não do campo do produto. O snapshot é informativo e recalculado a cada edição.
- **[Margem derivada negativa pode assustar o operador]** → Mitigado por aviso explícito com o `break_even_price` ("para margem positiva, o preço precisa ser ≥ R$ X") e pelo bloqueio apenas abaixo do CMV.
- **[Preços já cadastrados não são recalculados]** → Intencional (Non-Goal): o objetivo é justamente parar de mexer no preço sem ação do operador. O preço só muda quando o produto é editado e salvo.
- **[`final_price` exatamente igual ao CMV gera divisão coerente mas margem fortemente negativa]** → `min_price = CMV` é o piso aberto inferior do que se aceita; no piso a margem é `−(cv+df+ir)`. O aviso de margem negativa cobre a faixa CMV..break-even.
