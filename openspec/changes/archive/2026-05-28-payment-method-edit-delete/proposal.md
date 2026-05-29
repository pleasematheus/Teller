## Why

A tela de Formas de Pagamento não permite alterar o nome nem excluir um método cadastrado — apenas desativá-lo. Isso força o usuário a manter registros errados ou desnecessários na base, poluindo a lista e impedindo correções simples como erros de digitação.

## What Changes

- Adicionar opção de **editar o nome** de uma forma de pagamento já cadastrada
- Adicionar opção de **excluir** uma forma de pagamento (com confirmação)
- Novos Tauri commands: `rename_payment_method` e `delete_payment_method`
- A exclusão deve ser bloqueada se o método estiver vinculado a transações existentes, exibindo mensagem explicativa

## Capabilities

### New Capabilities

- Nenhuma nova capability — as mudanças expandem a capability existente

### Modified Capabilities

- `configuracoes-pagamentos`: adicionar requisitos de renomeação e exclusão de formas de pagamento

## Impact

- **Frontend**: `src/` — componente de listagem de formas de pagamento precisa de novos controles (botão renomear / excluir)
- **Backend Rust**: novo command `rename_payment_method(id, name)` e `delete_payment_method(id)` com validação de uso
- **DB**: nenhuma mudança de schema; apenas novos `UPDATE` e `DELETE` em `Payment_Methods`
