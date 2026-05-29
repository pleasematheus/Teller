## Context

A tela `PagamentosSection.tsx` exibe uma lista de métodos com toggle ativo/inativo e botão "Editar taxas". O backend em `settings.rs` expõe `create_payment_method`, `update_payment_method` (apenas campo `active`) e os commands de taxas. Não há como renomear ou excluir um método.

A tabela `Sale_Payments` referencia `payment_method_id` — excluir um método usado em vendas corromperia o histórico financeiro.

## Goals / Non-Goals

**Goals:**
- Permitir renomear o nome de uma forma de pagamento existente
- Permitir excluir uma forma de pagamento não usada em vendas, com confirmação
- Bloquear exclusão quando o método tem vendas associadas, com mensagem clara

**Non-Goals:**
- Não migrar vendas existentes para outro método ao excluir
- Não alterar o schema do banco de dados
- Não modificar o command `update_payment_method` existente (mantém contrato atual)

## Decisions

**D1: Dois novos commands ao invés de ampliar `update_payment_method`**
`update_payment_method` já tem contrato estável (`id`, `active`). Criar `rename_payment_method(id, name)` e `delete_payment_method(id)` mantém cada command com responsabilidade única e evita quebra de contrato.

**D2: Bloqueio de exclusão via verificação no backend**
O backend verifica `COUNT(*) FROM Sale_Payments WHERE payment_method_id = ?` antes de deletar. Retorna `Err` descritivo se houver registros. A restrição fica na camada que tem acesso ao dado, não no frontend.

**D3: Renomear inline no form-edit existente**
Em vez de criar um fluxo separado de renomeação, o `form-edit` já exibe o nome do método como heading. A mudança mais simples é tornar esse campo editável — o usuário já está no contexto do método ao editar taxas. Salvar as taxas também salva o nome se alterado.

**D4: Confirmação de exclusão no frontend, sem modal separado**
Um botão "Excluir" no `form-edit` que, ao clicar, exibe um estado de confirmação inline (substituindo o botão por "Confirmar exclusão" + "Cancelar"), sem abrir um modal/dialog. Mantém a stack de UI simples.

## Risks / Trade-offs

- [Renomear afeta vendas passadas] → O nome exibido em relatórios históricos pode mudar. Mitigation: aceitável, pois o ID é preservado e o vínculo financeiro não é afetado.
- [Exclusão bloqueada pode confundir] → Usuário tenta excluir e recebe erro sem alternativa. Mitigation: mensagem explicativa com contagem de vendas vinculadas.

## Migration Plan

Nenhuma migração de schema necessária. Novos commands são adicionais — nenhuma função existente é removida ou alterada.
