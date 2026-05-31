## ADDED Requirements

### Requirement: Modalidade de desconto por item no fechamento

O sistema SHALL permitir, na tela de fechamento, escolher para cada item a modalidade do desconto: porcentagem (%) ou valor absoluto (R$). A modalidade é estado de interface; o valor enviado ao backend SHALL ser sempre a fração de desconto (0-1) por item, independentemente da modalidade escolhida. A modalidade inicial de cada item SHALL ser porcentagem (%).

#### Scenario: Modalidade inicial em porcentagem
- **WHEN** a tela de fechamento carrega os itens do pedido
- **THEN** cada item exibe o campo de desconto em modo `%`, com o comportamento atual preservado (entrada em porcentagem convertida para fração `valor / 100`)

#### Scenario: Desconto por item em valor (R$)
- **WHEN** o operador alterna a modalidade de um item para `R$` e informa um valor absoluto `v`
- **THEN** o frontend calcula a fração `discount = clamp(v / (price × quantity), 0, 1)` para aquele item e usa essa fração no envio ao `close_sale`

#### Scenario: Desconto por item em porcentagem (%)
- **WHEN** o operador mantém ou alterna a modalidade de um item para `%` e informa um percentual `p`
- **THEN** o frontend calcula a fração `discount = clamp(p / 100, 0, 1)` para aquele item

#### Scenario: Linha com total zero no modo valor
- **WHEN** a modalidade do item é `R$` e `price × quantity <= 0`
- **THEN** o frontend mantém `discount = 0` para aquele item sem erro de divisão

#### Scenario: Alternância de modalidade preserva o desconto
- **WHEN** o operador alterna a modalidade de um item entre `%` e `R$` sem digitar novo valor
- **THEN** o desconto aplicado (fração armazenada) é preservado e reexibido convertido para a nova modalidade, sem alterar o `discounted_price` do item

#### Scenario: Indicador de máximo na modalidade ativa
- **WHEN** o campo de desconto de um item está focado OU `discount > 0`
- **THEN** o indicador de desconto máximo continua exibindo o limite em R$ e em % (`· máx. R$ X (Y%)`), com ênfase na unidade da modalidade ativa
