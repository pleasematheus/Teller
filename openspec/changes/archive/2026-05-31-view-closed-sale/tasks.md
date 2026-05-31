## 1. Backend: command get_sale_payments

- [x] 1.1 Em `src-tauri/src/sales.rs`, adicionar `#[derive(Serialize)] struct SalePaymentRow` com `payment_method_name: String`, `amount_paid: f64`, `net_amount: f64`
- [x] 1.2 Implementar `#[tauri::command] get_sale_payments(sale_id: i64, state: State<DbState>) -> Result<Vec<SalePaymentRow>, String>` com `SELECT pm.name, sp.amount_paid, sp.net_amount FROM "Sale_Payments" sp JOIN "Payment_Methods" pm ON pm.id = sp.payment_method_id WHERE sp.sale_id = ? ORDER BY sp.created_at`
- [x] 1.3 Registrar `get_sale_payments` no `invoke_handler` em `src-tauri/src/lib.rs`

## 2. Backend: testes

- [x] 2.1 Adicionar teste em `#[cfg(test)]` de `sales.rs`: após fechar uma venda com um pagamento, uma consulta a `Sale_Payments` JOIN `Payment_Methods` retorna o nome do método, `amount_paid` e `net_amount` esperados (espelhar o padrão dos testes de fechamento existentes)
- [x] 2.2 Adicionar teste: consulta de pagamentos de um pedido sem `Sale_Payments` retorna conjunto vazio
- [x] 2.3 Rodar `rtk cargo test` e confirmar que os testes existentes continuam passando

## 3. Frontend: tipo e componente SaleDetail

- [x] 3.1 Em `src/types/sale.ts`, adicionar `SalePaymentRow { payment_method_name: string; amount_paid: number; net_amount: number }`
- [x] 3.2 Criar `src/components/sales/SaleDetail.tsx`: recebe `sale: Sale`; carrega itens via `get_sale_items` e pagamentos via `get_sale_payments`; renderiza recibo read-only (itens com preço cheio, desconto aplicado e subtotal descontado; total da venda; pagamentos com forma + valor)
- [x] 3.3 Garantir que `SaleDetail` não possui campos editáveis nem botões de salvar/editar/fechar venda (apenas exibição; reusar formatação `brl`/`pct` do projeto)

## 4. Frontend: integração no histórico

- [x] 4.1 Em `src/components/sales/SalesList.tsx`, no `mode="history"`, adicionar botão "Ver" que chama `onView(sale)` mantendo a exibição da data
- [x] 4.2 Em `src/components/sales/SalesModule.tsx`, adicionar o `FormContent` `{ kind: "detail-sale"; sale: Sale }`, tratar em `formLabel` (ex.: `Pedido #N`), criar `openDetail(sale)` e passar como `onView` da `SalesList` do histórico
- [x] 4.3 Renderizar `<SaleDetail>` na área de form quando `formContent?.kind === "detail-sale"`, seguindo o padrão de visibilidade (`activeView === "form" ? ... : "hidden"`) e o botão × de fechar já existentes

## 5. Verificação

- [x] 5.1 Rodar `rtk bun run test` (Vitest) e `rtk tsc` para garantir tipos e testes ok
- [x] 5.2 [Teste manual — tester humano] Fechar um pedido com desconto e múltiplos pagamentos; no histórico, clicar "Ver" e conferir que o recibo mostra itens com desconto, total e as formas de pagamento, sem permitir edição
- [x] 5.3 [Teste manual — tester humano] Abrir o detalhe de um pedido fechado sem desconto e confirmar que os subtotais aparecem sem indicação de desconto
