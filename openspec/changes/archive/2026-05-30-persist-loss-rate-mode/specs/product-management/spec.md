# Spec Delta: product-management

## ADDED Requirements

### Requirement: Persistência do modo da taxa de perda

O sistema SHALL persistir o modo da taxa de perda (`loss_mode`, com valores `manual` ou `auto`) por produto, com default `manual`, e restaurar o modo selecionado ao reabrir o produto para edição. O modo é uma preferência salva: alterar o modo não recalcula nem altera `expected_loss_rate`; o valor só muda por entrada manual ou ao acionar "Calcular". Tanto o modo quanto o valor permanecem inalterados até a próxima edição salva.

#### Scenario: Persistir modo ao salvar

- **WHEN** o usuário salva um produto com `loss_mode = auto`
- **THEN** o sistema grava `loss_mode = auto` no registro do produto

#### Scenario: Restaurar modo na edição

- **WHEN** o usuário reabre para edição um produto salvo com `loss_mode = auto`
- **THEN** o formulário exibe o modo `auto` selecionado, sem reiniciar para `manual`

#### Scenario: Default manual

- **WHEN** um produto é criado sem `loss_mode` informado ou é lido de um registro legado sem o campo
- **THEN** o sistema assume `loss_mode = manual`

#### Scenario: Modo inválido rejeitado

- **WHEN** o usuário submete um `loss_mode` diferente de `manual` ou `auto`
- **THEN** o sistema rejeita com erro de validação

#### Scenario: Alterar modo não altera o valor

- **WHEN** o usuário troca `loss_mode` entre `manual` e `auto` sem acionar "Calcular" nem digitar
- **THEN** `expected_loss_rate` permanece com o valor atual, sem recálculo

## MODIFIED Requirements

### Requirement: Listagem de produtos

O sistema SHALL retornar todos os produtos cadastrados com seus dados de precificação, ordenados por nome, incluindo os campos `sellable`, `unit`, `profit_margin` e `loss_mode` para uso no cliente.

#### Scenario: Lista com produtos existentes
- **WHEN** o comando `list_products` é invocado
- **THEN** o sistema retorna todos os registros de `Products` com campos: id, name, description, active, sellable, cost_price, markup, final_price, expected_loss_rate, unit, profit_margin, **loss_mode**

#### Scenario: Lista vazia
- **WHEN** nenhum produto está cadastrado
- **THEN** o sistema retorna lista vazia sem erro

### Requirement: Cálculo automático de taxa de perda

O sistema SHALL calcular `expected_loss_rate` a partir do histórico de `Stock_Movements` do produto quando o usuário solicitar, como alternativa ao preenchimento manual. O cálculo SHALL ocorrer apenas mediante ação explícita do usuário ("Calcular"); o sistema NÃO recalcula `expected_loss_rate` automaticamente na leitura ou exibição, mesmo quando `loss_mode = auto`.

#### Scenario: Calcular perda a partir do histórico

- **WHEN** o usuário aciona "Calcular" para perda automática em um produto existente
- **THEN** o sistema retorna `SUM(quantity WHERE movement_type='out' AND motive='perda') / SUM(quantity WHERE movement_type='in')` para aquele produto

#### Scenario: Sem histórico de entradas

- **WHEN** o produto não possui movimentos de entrada (`SUM = 0`)
- **THEN** o sistema retorna taxa de perda 0 (evita divisão por zero)

#### Scenario: Modo auto indisponível em criação

- **WHEN** o formulário está em modo de criação (produto novo)
- **THEN** a opção de cálculo automático de perda está desabilitada (produto ainda não tem histórico)

#### Scenario: Valor congelado entre cálculos

- **WHEN** um produto está salvo com `loss_mode = auto` e novos movimentos de perda são registrados no estoque
- **THEN** `expected_loss_rate` permanece com o último valor salvo até o usuário acionar "Calcular" e salvar novamente
