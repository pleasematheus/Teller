## ADDED Requirements

### Requirement: Integridade de quantidade na entrada de compra
O formulário de compra SHALL impedir o registro de quantidade fracionada para itens cuja unidade é inteira (`Units_Of_Measure.integer = true`). A quantidade SHALL ser normalizada para inteiro durante a edição (arredondamento com piso mínimo de 1), de forma consistente com a tela de venda, e a quantidade fracionada SHALL ser bloqueada no submit como defesa secundária. Quando a conversão caixa -> unidade (`unit_qty`) estiver ativa e a unidade do produto for inteira, a quantidade de estoque resultante (`quantity * unit_qty`) SHALL também ser inteira; caso contrário o item SHALL ser rejeitado no submit. Itens com unidade fracionada (`integer = false`) SHALL continuar aceitando casas decimais, respeitando o piso mínimo positivo.

#### Scenario: Digitação fracionada em unidade inteira é normalizada
- **WHEN** o operador digita uma quantidade com casas decimais em um item cuja unidade é inteira e o campo perde o foco
- **THEN** o valor é normalizado para o inteiro mais próximo, com mínimo de 1
- **THEN** enquanto o valor permanecer fracionado, o campo exibe o aviso "Apenas inteiros"

#### Scenario: Submit bloqueado com quantidade fracionada em unidade inteira
- **WHEN** o operador tenta salvar a compra com algum item de unidade inteira e quantidade fracionada
- **THEN** o sistema não envia `create_purchase`/`update_purchase` e exibe mensagem orientando o uso de números inteiros

#### Scenario: Conversão caixa para unidade gera estoque fracionado em unidade inteira
- **WHEN** um item tem unidade de produto inteira, a conversão `unit_qty` ativa, e `quantity * unit_qty` resulta em valor fracionado
- **THEN** o sistema bloqueia o submit e sinaliza que a quantidade de estoque resultante deve ser inteira

#### Scenario: Unidade fracionada aceita casas decimais
- **WHEN** o operador digita uma quantidade com casas decimais em um item cuja unidade é fracionada (`integer = false`)
- **THEN** o valor é aceito sem normalização para inteiro, respeitando o piso mínimo positivo
