## Why

Hoje a precificação é dirigida pela margem: o operador informa `profit_margin` e o `final_price` é derivado por `CMV / (1 − (cv + df + investment_rate + margin))`. Como `cv`, `df` e `investment_rate` vêm do histórico agregado da empresa (`get_pricing_rates`), o preço de venda exibido/recalculado de cada produto muda toda vez que ocorre uma movimentação (venda, despesa, taxa de cartão). Isso torna o preço de etiqueta instável, quando na prática quem deveria flutuar é a margem (uma leitura analítica), não o preço cobrado do cliente.

## What Changes

- **Inversão do modelo de precificação**: o `final_price` passa a ser a entrada primária (definida pelo operador) e a `profit_margin` passa a ser **derivada** dele, via `margin = 1 − cv − df − investment_rate − CMV/final_price`. O preço de venda deixa de oscilar com o histórico da empresa.
- Nova função pura `computeMarginFromPrice` em `src/lib/pricing.ts` que, dado custo, taxa de perda, preço final e taxas, retorna `cmv`, `markup_percent`, `profit_margin` (derivada, podendo ser negativa), o piso de custo (`min_price = CMV`), o preço de equilíbrio (`break_even_price`) e indicadores `below_cost` / `feasible`.
- **Piso de custo (CMV) para impedir prejuízo**: o `final_price` não pode ser inferior ao `CMV` (custo já ajustado pela taxa de perda). Abaixo desse piso o formulário sinaliza prejuízo e bloqueia o salvamento; entre o CMV e o preço de equilíbrio a margem derivada é negativa e o sistema exibe aviso.
- O `ProductForm` passa a editar o campo **Preço final (R$)** e a exibir CMV, Markup% e **Margem derivada** como leitura. A sugestão automática (hoje pré-populando a margem com `get_realized_margin`) passa a pré-popular um **preço sugerido** derivado da margem realizada.
- A fórmula de markup divisor **não muda**; apenas se inverte qual variável é entrada e qual é derivada.

## Capabilities

### New Capabilities
<!-- Nenhuma capability nova -->

### Modified Capabilities
- `product-management`: a precificação automática no formulário passa a ser dirigida pelo preço final (margem derivada) em vez de dirigida pela margem; valida o piso de CMV para evitar prejuízo; substitui a validação dinâmica do campo de margem pela validação dinâmica do campo de preço.

## Impact

- `src/lib/pricing.ts`: nova função `computeMarginFromPrice`; `computePrice` mantida para gerar o preço sugerido inicial.
- `src/types/product.ts`: nova interface `MarginBreakdown` (cmv, markup_percent, profit_margin, min_price, break_even_price, below_cost, feasible).
- `src/lib/pricing.test.ts`: novos casos cobrindo margem derivada, piso de CMV, margem negativa e preço de equilíbrio.
- `src/components/products/ProductForm.tsx`: campo de entrada passa a ser preço final; margem vira leitura derivada; aviso/bloqueio de preço abaixo do custo; pré-população por preço sugerido.
- Sem mudança de schema: `Products` já persiste `final_price`, `markup` e `profit_margin`. Os comandos Tauri `create_product`/`update_product` já recebem os três campos; muda apenas qual deles é fonte de verdade no cliente. Sem breaking changes para dados persistidos.
