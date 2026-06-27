## Context

O fechamento bloqueia overpayment em dois pontos:

- Frontend `SaleCloseForm.tsx:148-152`: `remaining = discountedTotal - totalPaid`; `canSubmit` exige `Math.abs(remaining) < 0.01`.
- Backend `sales.rs:327-333`: `(total_paid - total_value).abs() > 0.01` retorna erro.

Métodos de pagamento são definidos pelo usuário apenas por `name` (`Payment_Methods` não tem flag de tipo). Não há como o sistema saber qual método é dinheiro.

## Decisões

### 1. Identificação de dinheiro: flag `is_cash`

Coluna `is_cash BOOLEAN NOT NULL DEFAULT false` em `Payment_Methods`. Robusto e independente do nome (o usuário pode chamar de "Dinheiro", "Espécie", etc.). Configurável na tela de pagamentos.

Alternativas descartadas: identificar por nome "Dinheiro" (frágil); permitir overpay em qualquer método (troco em cartão/Pix não faz sentido).

### 2. "Aplicado" vs "Recebido" — troco só na UI (Opção A)

```
 Venda R$48, cliente entrega R$50 em dinheiro
 ┌──────────────┬───────┐
 │ recebido     │  50   │  ← entrada do operador (apenas linha is_cash)
 │ aplicado     │  48   │  ← quita a venda; é o amount_paid enviado ao backend
 │ troco        │   2   │  ← recebido − aplicado; exibido, não persistido
 └──────────────┴───────┘
```

A soma dos **aplicados** continua igual ao total, então a validação do backend (`Σ amount_paid == total_value`) permanece correta **sem alteração**. `close_sale` e `Sale_Payments` não mudam. O troco é efeito de caixa neutro (sai da gaveta), não é receita.

Descartado (Opção B): persistir `amount_received`/`change_given` em `Sale_Payments` para conciliação de gaveta. Maior superfície (schema + backend + relatório); fica como mudança futura separada.

### 3. Modelo de cálculo (suporta múltiplas linhas e dinheiro misto)

Processa linhas em ordem, com `remaining` iniciando em `discountedTotal`:

```
para cada linha:
  se NÃO is_cash:
    aplicado = input            # comportamento atual
  se is_cash:
    aplicado = min(recebido, remaining)
    troco_linha = recebido - aplicado
  remaining -= aplicado

troco_total = Σ troco_linha (linhas is_cash)
```

Mantém a invariante `Σ aplicado == total` enquanto `Σ recebido(dinheiro) + Σ aplicado(não-dinheiro) >= total`. Caso comum (1 linha de dinheiro) cai naturalmente: aplicado = total, troco = recebido − total.

### 4. UX (b): campo único "Recebido"

Linha `is_cash` mostra apenas o campo "Recebido"; o aplicado é derivado (`min(recebido, remaining)`) e o troco é exibido abaixo. Menos digitação no fluxo de caixa comum. Linhas não-dinheiro permanecem com o campo de valor atual (= aplicado).

`canSubmit` passa a exigir:
- `remaining == 0` após processar todas as linhas (i.e. `Σ aplicado == total`);
- toda linha não-dinheiro com aplicado > 0;
- toda linha dinheiro com `recebido > 0` e `recebido >= aplicado` (garantido por `min`, mas validar `recebido >= aplicado` para feedback de insuficiência).

### 5. Migração de schema

`DB_VERSION` "7" → "8". Primeiro entry em `MIGRATIONS` (`db.rs:42`, hoje `&[]`):

```
MigrationStep targeting v8:
  Sql(ALTER TABLE "Payment_Methods" ADD COLUMN "is_cash" BOOLEAN NOT NULL DEFAULT false)
```

Nota: `MIGRATIONS` está vazio com `DB_VERSION="7"` (baseline v6) — o framework de migração nunca rodou em produção. Este será o primeiro step real; precisa de teste de migração v7→v8. `schema.sql` também ganha a coluna para instalações novas.

## Backend

- `list_payment_methods`: incluir `is_cash` no struct retornado.
- Novo command `set_payment_method_cash(id, is_cash, session_token)` espelhando `update_payment_method` (`settings.rs:322`): `UPDATE "Payment_Methods" SET is_cash = ?, updated_at = CURRENT_TIMESTAMP WHERE id = ?`. Registrar em `lib.rs`.
- `close_sale`: nenhuma mudança.

## Risks

- Múltiplas linhas de dinheiro: o modelo `min(recebido, remaining)` ordena-dependente, mas correto para a soma. Edge raro; documentado no spec.
- Migração inédita: cobrir com teste; incluir tarefa de recriar o banco local de desenvolvimento.
