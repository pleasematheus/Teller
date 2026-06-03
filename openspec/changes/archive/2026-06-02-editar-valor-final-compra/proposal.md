## Why

Na tela de compra, o operador registra quantidade e custo, mas não consegue definir ali o **preço de venda final** do produto (o que o cliente vai pagar). Hoje esse preço (`Products.final_price`) só é editável na tela de produto, separando o momento em que o custo muda (compra) do momento em que o preço de venda é revisto. Como o preço de venda é derivado do custo, faz sentido sugerir e permitir ajustar o preço de venda na própria compra. Além disso, o frete pago hoje não entra no custo do insumo, então o preço sugerido ignora o frete.

## What Changes

- Adicionar, por item da compra, um campo de **preço de venda editável** (o que o cliente paga), pré-preenchido com o **preço sugerido** pela fórmula de precificação já existente no sistema (`computePrice`), que o operador pode sobrescrever.
- Ao salvar a compra, `create_purchase` SHALL persistir o `final_price` informado em `Products` e derivar `markup`/`profit_margin` como snapshot (mesma regra da tela de produto), usando o `cost_price` resultante da compra.
- **BREAKING** (comportamento): ratear o frete entre os itens proporcionalmente ao valor de cada linha e incluí-lo no custo do insumo (`cost_price`), de modo que o frete passe a integrar o custo que alimenta o preço de venda sugerido. Isso inverte o comportamento atual em que o frete não entrava no custo do insumo.

## Capabilities

### New Capabilities
<!-- Nenhuma capability nova: compra, custo e precificação já são cobertos pelas capabilities existentes. -->

### Modified Capabilities
- `stock-management`: o requisito de registro de compra multi-item passa a expor um preço de venda sugerido e editável por item, e `create_purchase` passa a persistir `final_price` (com `markup`/`profit_margin` derivados) no produto; o requisito de custo médio ponderado passa a incluir a parcela de frete rateada no custo unitário do lote, substituindo o cenário atual em que o frete não entra no custo do insumo.

## Impact

- Frontend: `src/components/purchases/PurchaseForm.tsx` (buscar `get_pricing_rates`, calcular preço sugerido por item via `computePrice`, campo de preço de venda editável com override). `src/types/product.ts` (`PurchaseItemPayload` ganha `final_price`).
- Backend: `src-tauri/src/stock.rs` — `PurchaseItemInput` ganha `final_price`; `do_purchase_work` rateia frete no custo, persiste `final_price` em `Products` e deriva `markup`/`profit_margin` a partir do `cost_price` resultante e da `expected_loss_rate`. Testes em `#[cfg(test)]`.
- Precificação: o preço sugerido reaproveita `src/lib/pricing.ts` (`computePrice`); a derivação de `markup`/`profit_margin` no backend espelha `computeMarginFromPrice` / a regra de snapshot do `product-management`.
- Banco de dados: sem alteração de schema — `final_price`, `markup`, `profit_margin`, `unit_cost` e `freight` já existem.
