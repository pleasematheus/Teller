## Why

No formulário de cadastro de produto, quando `CV% + DF% + Invest% + Margem%` atinge ou ultrapassa 100% o denominador do markup divisor fica ≤ 0 e o sistema exibe **Preço final: R$ 0,00 silenciosamente**. O usuário interpreta isso como um cálculo bugado, quando na verdade a margem informada é matematicamente inviável para o modelo "margem por dentro". O campo de margem ainda agrava o problema com um `max={99}` cosmético que não impede a digitação de valores inválidos.

A fórmula de markup divisor está **correta** e não muda — o que falta é comunicar a inviabilidade ao usuário e impedir entradas que quebram o modelo.

## What Changes

- `computePrice` passa a sinalizar inviabilidade no `PriceBreakdown`: novos campos indicando que a combinação de taxas+margem é inviável (`feasible: false`) e qual a margem máxima possível (`max_margin = 1 − cv − df − investment_rate`).
- O `ProductForm` exibe um aviso explícito — "Margem inviável: máximo possível ≈ Y%" — no lugar do `R$ 0,00` silencioso quando o cálculo é inviável.
- O input de margem ganha validação dinâmica: `max` derivado de `(1 − cv − df − invest)` em vez do `max={99}` fixo, com clamp visual coerente.
- A fórmula de precificação **não muda**; apenas o retorno de `computePrice` ganha metadados de viabilidade.

## Capabilities

### New Capabilities
<!-- Nenhuma capability nova -->

### Modified Capabilities
- `product-management`: a precificação automática no formulário passa a detectar e comunicar margem inviável (em vez de exibir preço zero sem explicação) e a restringir dinamicamente o valor de margem aceito ao máximo viável.

## Impact

- `src/lib/pricing.ts`: `computePrice` retorna campos adicionais (`feasible`, `max_margin`) no `PriceBreakdown`.
- `src/types/product.ts`: interface `PriceBreakdown` ganha os novos campos.
- `src/lib/pricing.test.ts`: novos casos cobrindo viabilidade e cálculo de margem máxima.
- `src/components/products/ProductForm.tsx`: aviso de inviabilidade e `max` dinâmico no campo de margem.
- Sem mudança de schema, de comandos Tauri ou da fórmula de cálculo. Sem breaking changes para dados persistidos.
