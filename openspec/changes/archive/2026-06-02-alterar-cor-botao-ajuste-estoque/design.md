## Context

O botão "Ajuste" na tabela de saldo de estoque (`StockBalance.tsx`) usa `variant="outline"`. Esse variant tem fundo transparente e borda sutil — visualmente passivo, parece texto com borda, não ação clicável. O projeto usa `variant="secondary"` para botões de ação secundária com fundo sólido.

## Goals / Non-Goals

**Goals:**
- Botão "Ajuste" comunica visualmente que é clicável
- Consistência com padrão já usado em outros botões de ação na listagem (ex: EntradaList, FinanceiroList)

**Non-Goals:**
- Não alterar tamanho, texto, posição ou lógica do botão
- Não revisar outros botões `outline` no projeto

## Decisions

**Trocar `variant="outline"` por `variant="secondary"`**

`secondary` = `bg-secondary text-secondary-foreground` — fundo sólido, claramente distinguível das células da tabela. Alternativa `default` (primary) seria muito proeminente para uma ação por linha em tabela.

## Risks / Trade-offs

Nenhum risco. Mudança de um prop, sem impacto em lógica ou testes.
