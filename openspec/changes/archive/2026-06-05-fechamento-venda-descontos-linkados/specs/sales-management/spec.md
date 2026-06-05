## MODIFIED Requirements

### Requirement: Modalidade de desconto por item no fechamento

O sistema SHALL permitir, na tela de fechamento, escolher a modalidade do desconto: porcentagem (%) ou valor absoluto (R$). A modalidade é um único estado de interface compartilhado por toda a tela: SHALL existir um único seletor `R$ / %` que controla simultaneamente a exibição do campo de desconto de todos os itens e do desconto global. O valor enviado ao backend SHALL ser sempre a fração de desconto (0-1) por item, independentemente da modalidade escolhida. A modalidade inicial SHALL ser porcentagem (%).

#### Scenario: Modalidade inicial em porcentagem
- **WHEN** a tela de fechamento carrega os itens do pedido
- **THEN** o seletor compartilhado está em modo `%` e todos os campos de desconto (itens e global) exibem em `%`, com a entrada em porcentagem convertida para fração `valor / 100`

#### Scenario: Seletor único controla toda a tela
- **WHEN** o operador alterna o seletor de modalidade da tela entre `%` e `R$`
- **THEN** os campos de desconto de todos os itens e o campo de desconto global passam imediatamente a refletir a nova modalidade, sem seletores de modalidade independentes por item

#### Scenario: Desconto por item em valor (R$)
- **WHEN** a modalidade da tela é `R$` e o operador informa um valor absoluto `v` no campo de um item
- **THEN** o frontend calcula a fração `discount = clamp(v / (price × quantity), 0, 1)` para aquele item e usa essa fração no envio ao `close_sale`

#### Scenario: Desconto por item em porcentagem (%)
- **WHEN** a modalidade da tela é `%` e o operador informa um percentual `p` no campo de um item
- **THEN** o frontend calcula a fração `discount = clamp(p / 100, 0, 1)` para aquele item

#### Scenario: Linha com total zero no modo valor
- **WHEN** a modalidade da tela é `R$` e `price × quantity <= 0` para um item
- **THEN** o frontend mantém `discount = 0` para aquele item sem erro de divisão

#### Scenario: Alternância de modalidade preserva o desconto
- **WHEN** o operador alterna a modalidade da tela entre `%` e `R$` sem digitar novo valor
- **THEN** o desconto aplicado (fração armazenada) de cada item é preservado e reexibido convertido para a nova modalidade, sem alterar o `discounted_price` dos itens

#### Scenario: Indicador de máximo na modalidade ativa
- **WHEN** o campo de desconto de um item está focado OU `discount > 0`
- **THEN** o indicador de desconto máximo continua exibindo o limite em R$ e em % (`· máx. R$ X (Y%)`), com ênfase na unidade da modalidade ativa da tela

### Requirement: Desconto global rateado

O sistema SHALL permitir, na tela de fechamento, aplicar um desconto no total do pedido, distribuído proporcionalmente pelo valor de cada item. O campo de desconto global SHALL usar a mesma modalidade do seletor único compartilhado da tela.

#### Scenario: Desconto global em R$
- **WHEN** a modalidade da tela é `R$` e o operador informa um valor de desconto global
- **THEN** o frontend calcula `peso_i = (price_i × qty_i) / total_bruto` e `discount_pct_i = (desconto_global × peso_i) / (price_i × qty_i)` para cada item, atualizando os descontos individuais

#### Scenario: Desconto global em %
- **WHEN** a modalidade da tela é `%` e o operador informa um percentual de desconto global
- **THEN** o frontend aplica o mesmo percentual a todos os itens (`discount_pct_i = percentual`)

#### Scenario: Rateio com item abaixo do piso
- **WHEN** o rateio do desconto global resulta em ao menos um item com `discounted_price < floor`
- **THEN** o sistema exibe aviso identificando quais itens ultrapassaram o piso, sem bloquear o fechamento

#### Scenario: Alternar modalidade limpa o campo global
- **WHEN** o operador alterna o seletor de modalidade da tela
- **THEN** o campo de entrada do desconto global é limpo, enquanto as frações de desconto já aplicadas aos itens são preservadas
