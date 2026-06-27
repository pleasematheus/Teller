## ADDED Requirements

### Requirement: Marcar método de pagamento como dinheiro

O sistema SHALL permitir que o usuário marque um método de pagamento como dinheiro (`is_cash`), atributo que habilita o cálculo de troco no fechamento. O atributo SHALL ser persistido em `Payment_Methods.is_cash` e retornado por `list_payment_methods`.

#### Scenario: Marcar método como dinheiro
- **WHEN** o usuário ativa a opção "dinheiro" de um método na tela de pagamentos
- **THEN** o sistema invoca `set_payment_method_cash(id, true)`, persiste `is_cash = true` e a tela reflete o estado

#### Scenario: Desmarcar método como dinheiro
- **WHEN** o usuário desativa a opção "dinheiro" de um método
- **THEN** o sistema invoca `set_payment_method_cash(id, false)` e persiste `is_cash = false`

#### Scenario: Listagem inclui o atributo
- **WHEN** `list_payment_methods` é invocado
- **THEN** cada método retornado inclui o campo `is_cash`

#### Scenario: Default para métodos existentes
- **WHEN** a coluna `is_cash` é adicionada via migração a métodos já cadastrados
- **THEN** os métodos existentes recebem `is_cash = false` por padrão
