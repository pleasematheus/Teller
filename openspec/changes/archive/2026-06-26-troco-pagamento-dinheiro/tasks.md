## 1. Schema e migração (DuckDB)

- [x] 1.1 Adicionar `"is_cash" BOOLEAN NOT NULL DEFAULT false` à definição de `Payment_Methods` em `src/db/schema.sql`
- [x] 1.2 Incrementar `DB_VERSION` de `"7"` para `"8"` em `src-tauri/src/db.rs`
- [x] 1.3 Adicionar o primeiro entry em `MIGRATIONS` (`db.rs:42`) direcionado à v8: `MigrationKind::Sql(r#"ALTER TABLE "Payment_Methods" ADD COLUMN "is_cash" BOOLEAN NOT NULL DEFAULT false"#)`
- [x] 1.4 Tarefa de DB (CLAUDE.md): deletar o banco de desenvolvimento local antigo e recriar a partir do schema novo após a mudança

## 2. Backend — Rust

- [x] 2.1 Incluir `is_cash` no struct/SELECT de `list_payment_methods` (`settings.rs`)
- [x] 2.2 Implementar `set_payment_method_cash(id: i64, is_cash: bool, session_token, state, session)` em `settings.rs` espelhando `update_payment_method`: `require_session`, `UPDATE "Payment_Methods" SET is_cash = ?, updated_at = CURRENT_TIMESTAMP WHERE id = ?`, `Err` se 0 linhas
- [x] 2.3 Registrar `set_payment_method_cash` no `tauri::Builder` em `lib.rs`
- [x] 2.4 Confirmar que `close_sale` permanece inalterado (validação `Σ amount_paid == total_value` sobre valores aplicados)

## 3. Testes Rust

- [x] 3.1 Teste de migração: banco em v7 ganha coluna `is_cash` (default false) e carimba v8
- [x] 3.2 Teste: `set_payment_method_cash` define `is_cash = true`/`false` com sucesso
- [x] 3.3 Teste: `set_payment_method_cash` retorna `Err` para id inexistente
- [x] 3.4 Teste: `list_payment_methods` retorna `is_cash`

## 4. Frontend — tipos e lógica de troco

- [x] 4.1 Adicionar `is_cash: boolean` a `PaymentMethod` em `src/types/sale.ts`
- [x] 4.2 Extrair em `src/lib/` função pura de cálculo de pagamento: dado `total`, linhas `{ aplicado | recebido, is_cash }` em ordem, retornar `{ remainingPorLinha, aplicadoPorLinha, trocoPorLinha, trocoTotal, remaining }` usando `aplicado = is_cash ? min(recebido, remaining) : input`
- [x] 4.3 `tests/lib/<nome>.test.ts` (Vitest, alias `@/lib/`): cobrir pagamento único com troco, recebido = total (troco 0), recebido insuficiente, split dinheiro + não-dinheiro, linha não-dinheiro inalterada — usar os `#### Scenario:` do spec como referência

## 5. Frontend — SaleCloseForm.tsx

- [x] 5.1 Para linhas cujo método tem `is_cash = true`, renderizar campo "Recebido" no lugar do valor; demais linhas mantêm o campo de valor aplicado
- [x] 5.2 Calcular aplicado/troco via a função pura de 4.2; exibir o troco da linha e/ou troco total
- [x] 5.3 Ajustar `remaining`/`canSubmit` (`SaleCloseForm.tsx:148-152`) para usar valores aplicados; habilitar fechamento quando `Σ aplicado == total` e cada linha de dinheiro tiver `recebido >= aplicado`
- [x] 5.4 Em `handleSubmit`, enviar `amount_paid` = valor aplicado (não recebido) por linha

## 6. Frontend — PagamentosSection.tsx

- [x] 6.1 Adicionar controle (Switch) "Dinheiro" por método que invoca `set_payment_method_cash(id, valor)` e atualiza o estado local
- [x] 6.2 Refletir `is_cash` ao carregar/listar métodos

## 7. Testes manuais (delegados a testador humano)

- [x] 7.1 [MANUAL TESTER] Marcar um método como dinheiro nas configurações e confirmar persistência após reabrir o app
- [x] 7.2 [MANUAL TESTER] Fechar venda em dinheiro com valor recebido > total: conferir troco exibido e venda fechada com `amount_paid` = total
- [x] 7.3 [MANUAL TESTER] Fechar venda split (cartão + dinheiro com troco) e conferir habilitação correta
- [x] 7.4 [MANUAL TESTER] Verificar que método não-dinheiro continua bloqueando overpayment
