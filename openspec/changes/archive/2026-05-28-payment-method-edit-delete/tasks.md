## 1. Backend — Rust commands

- [x] 1.1 Implementar `rename_payment_method(id: i64, name: String)` em `settings.rs`: validar nome não-vazio, executar `UPDATE Payment_Methods SET name = ?, updated_at = CURRENT_TIMESTAMP WHERE id = ?`
- [x] 1.2 Implementar `delete_payment_method(id: i64)` em `settings.rs`: verificar `COUNT(*) FROM Sale_Payments WHERE payment_method_id = ?`; retornar `Err` descritivo se > 0; caso contrário executar `DELETE FROM Payment_Methods WHERE id = ?`
- [x] 1.3 Registrar os dois novos commands no `tauri::Builder` em `lib.rs`

## 2. Testes Rust

- [x] 2.1 Teste unitário: `rename_payment_method` atualiza nome com sucesso
- [x] 2.2 Teste unitário: `rename_payment_method` rejeita nome vazio
- [x] 2.3 Teste unitário: `delete_payment_method` remove método sem vendas vinculadas
- [x] 2.4 Teste unitário: `delete_payment_method` retorna `Err` quando método tem registros em `Sale_Payments`

## 3. Frontend — PagamentosSection.tsx

- [x] 3.1 Tornar o campo de nome no `form-edit` editável: substituir o `<h2>` com o nome por um `<Input>` pré-preenchido; incluir o nome no estado local do form-edit
- [x] 3.2 Ao salvar taxas (`handleSaveFees`), verificar se o nome foi alterado e chamar `rename_payment_method` antes de salvar as taxas; exibir erro se a renomeação falhar
- [x] 3.3 Adicionar botão "Excluir" no `form-edit` que alterna para estado de confirmação inline (troca o botão por "Confirmar exclusão" + "Cancelar")
- [x] 3.4 Ao confirmar exclusão, chamar `delete_payment_method`; em caso de sucesso voltar para a lista e recarregar; em caso de `Err` exibir a mensagem de erro recebida do backend
