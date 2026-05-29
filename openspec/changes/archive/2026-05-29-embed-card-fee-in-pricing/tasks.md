## 1. Schema e migração

- [x] 1.1 Adicionar `installments INTEGER NOT NULL DEFAULT 1` em `Transaction_Fees` (`src/db/schema.sql`)
- [x] 1.2 Adicionar `applied_fee_percent DECIMAL(5,4) NOT NULL DEFAULT 0` e `applied_fixed_fee DECIMAL(10,2) NOT NULL DEFAULT 0` em `Sale_Payments` (`src/db/schema.sql`)
- [x] 1.3 Migração idempotente para bancos existentes (ALTER TABLE no fluxo de abertura resiliente em `db.rs`)
- [x] 1.4 Normalizar `Transaction_Fees` legadas: derivar `installments` da `description` (parse de "Nx"; default 1)

## 2. Backend — taxas por parcela (settings.rs)

- [x] 2.1 Ajustar struct `TransactionFee` para incluir `installments` (manter `fee_percent`, `fixed_fee`; `description` opcional)
- [x] 2.2 `upsert_transaction_fee`: receber `installments`, validar `>= 1` e unicidade por `(payment_method_id, installments)`
- [x] 2.3 `list_transaction_fees`: ordenar por `installments`
- [x] 2.4 Testes `#[cfg(test)]`: upsert cria/atualiza por parcela, rejeita `installments < 1`, rejeita duplicata; list ordenado por parcela

## 3. Backend — listagem de métodos (settings.rs)

- [x] 3.1 `list_payment_methods`: remover `max_fee_percent`; retornar a tabela de taxas (`installments`, `fee_percent`, `fixed_fee`) por método
- [x] 3.2 Ajustar struct `PaymentMethod` (sai `max_fee_percent`, entra a lista de taxas)
- [x] 3.3 Testes `#[cfg(test)]`: método com tabela retorna as taxas; método sem taxa retorna lista vazia

## 4. Backend — taxa embutida no cv (products.rs)

- [x] 4.1 Definir critério de "cartão" = método com ao menos uma `Transaction_Fee` (taxa > 0)
- [x] 4.2 `get_pricing_rates`: somar `Σ(amount_paid − net_amount)` dos pagamentos em cartão (últimos 12 meses) ao numerador do `cv`
- [x] 4.3 Fallback de referência quando não há histórico de cartão (à vista ou maior taxa — resolver Open Question; default conservador)
- [x] 4.4 Testes `#[cfg(test)]`: cv com taxas de cartão; cv com fallback sem histórico; sem cartão/sem taxa → parcela de cartão = 0

## 5. Backend — fechamento por parcela (sales.rs)

- [x] 5.1 `SalePaymentInput`: adicionar `installments`
- [x] 5.2 `do_close_sale_work`: buscar taxa por `(payment_method_id, installments)`; `net_amount = amount_paid × (1 − fee_percent) − fixed_fee`; gravar snapshot `applied_fee_percent`/`applied_fixed_fee`; remover o `MAX(fee_percent)`
- [x] 5.3 Método/parcela sem taxa → `fee_percent = 0`, `fixed_fee = 0`, `net_amount = amount_paid`
- [x] 5.4 Testes `#[cfg(test)]`: net_amount com % + fixo; split com parcelas diferentes; método sem taxa preserva o valor

## 6. Frontend — tipos e precificação (lib)

- [x] 6.1 Atualizar tipos TS: `PaymentMethod` com lista de taxas e `SalePaymentInput` com `installments` (`types/sale.ts`); remover `max_fee_percent`
- [x] 6.2 `pricing.ts` `computeMaxDiscount`: remover o termo `maxFeePct` do piso (passar a usar `cv + df`); atualizar assinatura
- [x] 6.3 Atualizar `pricing.test.ts` (piso sem `max_fee_pct`) e manter a suíte verde

## 7. Frontend — configuração de pagamentos (PagamentosSection.tsx)

- [x] 7.1 Trocar a UI de taxas por uma tabela indexada por parcela (`installments` em vez de descrição livre), com validação de duplicata e `installments >= 1`
- [x] 7.2 Ajustar chamadas a `upsert_transaction_fee` / `list_transaction_fees` para o novo formato

## 8. Frontend — fechamento (SaleCloseForm.tsx)

- [x] 8.1 Exibir seletor de parcela por linha quando o método tiver tabela, com a taxa visível em cada opção
- [x] 8.2 Enviar `installments` no `close_sale`; método sem parcelamento usa `installments = 1`
- [x] 8.3 Exibir "você recebe" (net por linha) de forma discreta

## 9. Verificação

- [x] 9.1 `cargo test` verde (sales, products, settings, reports)
- [x] 9.2 `bun run test` verde (pricing, dre)
- [x] 9.3 `cargo build` e `tsc` sem erros
- [x] 9.4 Fluxo manual: cadastrar tabela de taxas, fechar venda parcelada, conferir `net_amount` e os valores da DRE