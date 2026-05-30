## Context

A precificação do Teller usa o método de **markup divisor** ("margem por dentro"), onde CV%, DF%, Investimento% e Margem% são frações do preço de venda:

```
preço = CMV / (1 − (cv + df + investment_rate + margin))
```

Isso é matematicamente correto: como a taxa de cartão e os impostos escalam com o preço, o preço precisa ser resolvido algebricamente. A consequência é que a margem é **limitada**: conforme `cv + df + invest + margin → 1`, o preço tende ao infinito, e ao atingir/ultrapassar 1 o cálculo perde sentido.

Hoje `computePrice` (`src/lib/pricing.ts:17`) já detecta esse caso (`denominator <= 0`) e retorna `final_price: 0`, mas **sem nenhum sinal de inviabilidade**. O `ProductForm` apenas renderiza `R$ 0,00`, e o input de margem tem um `max={99}` que não impede digitação (`<input type="number">` não bloqueia valores acima de `max`). O resultado é que o usuário enxerga um "bug de cálculo".

## Goals / Non-Goals

**Goals:**
- Comunicar claramente quando a margem informada é inviável para o modelo, indicando a margem máxima possível.
- Impedir/restringir entradas inválidas no campo de margem de forma dinâmica.
- Manter `computePrice` uma função pura e testável, apenas enriquecendo seu retorno.

**Non-Goals:**
- Alterar a fórmula de precificação (markup divisor permanece).
- Mudar a semântica de "margem" para markup-sobre-custo.
- Mudar schema do banco, comandos Tauri ou dados persistidos.
- Exibir equivalência em markup-sobre-custo (fora de escopo desta mudança).

## Decisions

**1. Enriquecer `PriceBreakdown` com `feasible` e `max_margin` em vez de lançar erro.**
`computePrice` continua retornando um objeto; adiciona `feasible: boolean` e `max_margin: number` (= `1 − cv − df − investment_rate`, com piso em 0). Quando inviável, `final_price` continua `0` (preserva o contrato atual para qualquer consumidor existente) e `feasible: false`. Alternativa considerada: lançar exceção — descartada por quebrar a pureza/uso em `useMemo` e exigir try/catch em todo consumidor.

**2. `max_margin` é a margem máxima estritamente viável.** Como o preço tende ao infinito ao se aproximar de `max_margin`, o valor exato `max_margin` ainda é inviável (`denominator = 0`). A UI comunica "máximo possível ≈ Y%" como referência prática; a validação do input usa `max_margin` como teto aberto (margem deve ser `< max_margin`).

**3. Aviso na UI substitui o bloco de preço quando inviável.** Quando `!feasible`, o `ProductForm` mostra a mensagem "Margem inviável: CV+DF+Invest já consomem X% do preço; máximo possível ≈ Y%" no lugar da linha "Preço final R$ 0,00", evitando a leitura de "bug".

**4. `max` dinâmico no input de margem.** O atributo `max={99}` passa a ser `max = floor(max_margin × 100)` (em pontos percentuais). Como `max` nativo não bloqueia digitação, a fonte de verdade do feedback é o aviso de inviabilidade (decisão 3); o `max` dinâmico apenas melhora a semântica e a validação nativa de submit.

## Risks / Trade-offs

- **[Consumidores do `PriceBreakdown` podem ignorar os novos campos]** → Campos são aditivos e opcionais na prática; `final_price: 0` mantém o comportamento legado, então nada quebra se um consumidor não ler `feasible`.
- **[`max_margin` próximo de zero quando CV+DF+Invest é alto]** → É o comportamento correto: o aviso explica que as taxas já consomem o preço. Cobrir com teste de cenário.
- **[Confusão entre "máximo possível" exibido e teto real (assintótico)]** → Mensagem usa "≈" e a validação trata `max_margin` como limite aberto; documentado nos cenários do spec.
