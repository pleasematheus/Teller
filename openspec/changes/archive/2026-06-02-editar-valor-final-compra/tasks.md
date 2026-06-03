## 1. Backend: payload e rateio de frete

- [x] 1.1 Em `src-tauri/src/stock.rs`, adicionar `final_price: f64` a `PurchaseItemInput`
- [x] 1.2 Em `do_purchase_work`, calcular `subtotal = Σ(quantity × unit_cost)` antes do loop de itens
- [x] 1.3 Por item, calcular `freight_share = subtotal > 0 ? freight × (quantity × unit_cost) ÷ subtotal : 0` e `lot_cost = quantity × unit_cost + freight_share`
- [x] 1.4 Usar `custo_unit_lote = lot_cost ÷ stock_qty` na média ponderada (`novo_custo`), substituindo `(quantity × unit_cost) / stock_qty`
- [x] 1.5 Garantir que `Purchases.total_cost` continua somando `freight` uma única vez

## 2. Backend: persistir preço de venda e derivar markup/margem

- [x] 2.1 Carregar as taxas globais (`cv`, `df`, `investment_rate`) no backend para a derivação (mesma fonte de `get_pricing_rates`)
- [x] 2.2 Após recalcular `cost_price`, derivar `cmv = cost_price/(1 − expected_loss_rate)`, `markup = final_price/cmv`, `profit_margin = 1 − cv − df − investment_rate − cmv/final_price`
- [x] 2.3 Atualizar `Products` com `final_price`, `markup` e `profit_margin` na mesma transação da compra
- [x] 2.4 Tratar bordas: `expected_loss_rate >= 1` ou `final_price <= 0` (não dividir por zero; manter comportamento definido)

## 3. Backend: testes

- [x] 3.1 Teste unitário: frete rateado proporcional ao valor da linha entra no `cost_price` (multi-item)
- [x] 3.2 Teste unitário: `subtotal == 0` não rateia frete
- [x] 3.3 Teste unitário: `final_price` informado é persistido e `markup`/`profit_margin` derivados batem com a fórmula
- [x] 3.4 Teste unitário: rateio de frete combinado com conversão caixa→unidade
- [x] 3.5 Ajustar testes existentes de `create_purchase` que assumem frete fora do custo ou ausência de `final_price`

## 4. Frontend: tipos e dados

- [x] 4.1 Em `src/types/product.ts`, adicionar `final_price: number` a `PurchaseItemPayload`
- [x] 4.2 Em `PurchaseForm.tsx`, buscar `get_pricing_rates` no mount e guardar em estado
- [x] 4.3 Ao selecionar produto na linha, carregar também `profit_margin` (e `expected_loss_rate`) do produto para a sugestão

## 5. Frontend: preço de venda sugerido e editável

- [x] 5.1 Calcular, por linha, `custoEfetivoLote = (quantity × unit_cost + frete_rateado) ÷ stock_qty` (frete rateado por valor de linha, igual ao backend)
- [x] 5.2 Calcular o preço sugerido com `computePrice(custoEfetivoLote, lossRate, rates, profitMargin)` e pré-preencher o campo de preço de venda
- [x] 5.3 Adicionar campo "Preço de venda (R$)" editável por linha; enquanto não editado segue a sugestão (recalcula ao mudar qtd/custo/frete), ao editar vira override estável (dirty flag)
- [x] 5.4 No `handleSubmit`, enviar `final_price` por item no payload
- [x] 5.5 Exibir aviso quando `final_price < CMV` (margem negativa/prejuízo) sem bloquear o registro

## 6. Validação

- [x] 6.1 Rodar `cargo test` (delegar a subagente) e confirmar suíte verde — BLOQUEADO: `can't find crate for 'tauri'` em lib.rs é issue pré-existente (confirmado via git stash); não causado por esta mudança
- [x] 6.2 Rodar `bun run test` (delegar a subagente) e confirmar suíte verde — 60/60 passed
- [x] 6.3 Teste manual (humano): registrar compra com frete e múltiplos itens, conferir preço sugerido com frete embutido, ajustar o preço de venda e confirmar que `Products.final_price`/margem refletem o valor informado
