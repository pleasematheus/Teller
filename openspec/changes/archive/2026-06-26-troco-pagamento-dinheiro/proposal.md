## Why

No fechamento de venda (`SaleCloseForm`), o pagamento em dinheiro não permite devolver troco: a validação exige que a soma dos valores pagos seja exatamente igual ao total (`Math.abs(remaining) < 0.01` no frontend e `(total_paid - total_value).abs() > 0.01` no backend). Quando o cliente entrega um valor em dinheiro maior que o total, o operador não tem como registrar o valor recebido nem visualizar o troco a devolver. (issue #3)

## What Changes

- Adiciona uma flag `is_cash` por método de pagamento, identificando quais métodos representam dinheiro físico (e portanto podem gerar troco).
- Na configuração de pagamentos, o usuário pode marcar/desmarcar um método como dinheiro.
- No fechamento, linhas de pagamento de um método `is_cash` exibem um campo "Recebido" (valor entregue pelo cliente). O valor **aplicado** (que quita a venda) é derivado automaticamente; o **troco** = recebido − aplicado é exibido.
- A soma dos valores **aplicados** continua precisando igualar o total (invariante inalterada). O excedente em dinheiro vira troco, não inflaciona a receita.
- Troco é calculado e exibido apenas na UI. Não há persistência de valor recebido nem de troco (Opção A): `close_sale` e `Sale_Payments` permanecem inalterados, recebendo os valores aplicados.

## Capabilities

### Modified Capabilities

- `sales-management`: o fechamento passa a permitir overpayment em linhas de dinheiro, com cálculo e exibição de troco; a invariante de validação passa a ser sobre o valor aplicado.
- `configuracoes-pagamentos`: método de pagamento ganha atributo `is_cash`, configurável pelo usuário e retornado por `list_payment_methods`.
- `atualizacao-banco`: nova coluna `is_cash` em `Payment_Methods` exige bump de `DB_VERSION` e step de migração.

## Impact

- **Frontend**: `SaleCloseForm.tsx` (campo "Recebido", cálculo de aplicado/troco, ajuste de `canSubmit`); `PagamentosSection.tsx` (toggle `is_cash`); `types/sale.ts` (`PaymentMethod.is_cash`).
- **Backend**: `list_payment_methods` retorna `is_cash`; novo command `set_payment_method_cash(id, is_cash)` em `settings.rs`; registro em `lib.rs`. `close_sale` inalterado.
- **Schema**: `Payment_Methods` ganha `"is_cash" BOOLEAN NOT NULL DEFAULT false`.
- **DB**: `DB_VERSION` "7" → "8" + primeiro step em `MIGRATIONS` (`db.rs`) com `ALTER TABLE "Payment_Methods" ADD COLUMN ...`.
- **Dependências**: nenhuma nova.
