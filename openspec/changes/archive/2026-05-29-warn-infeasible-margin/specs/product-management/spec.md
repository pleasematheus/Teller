## ADDED Requirements

### Requirement: Detecção e aviso de margem inviável

O sistema SHALL detectar quando a combinação `cv + df + investment_rate + profit_margin` é maior ou igual a 1 (denominador do markup ≤ 0) e SHALL comunicar a inviabilidade ao usuário, indicando a margem máxima possível, em vez de exibir um preço final igual a zero sem explicação. A função de cálculo `computePrice` SHALL retornar, no `PriceBreakdown`, um indicador de viabilidade (`feasible`) e a margem máxima possível (`max_margin = max(0, 1 − cv − df − investment_rate)`).

#### Scenario: Margem viável retorna feasible verdadeiro
- **WHEN** `cv + df + investment_rate + profit_margin < 1`
- **THEN** `computePrice` retorna `feasible: true`, `final_price > 0` e `max_margin = max(0, 1 − cv − df − investment_rate)`

#### Scenario: Margem inviável retorna feasible falso
- **WHEN** `cv + df + investment_rate + profit_margin >= 1`
- **THEN** `computePrice` retorna `feasible: false`, `final_price: 0` e `max_margin = max(0, 1 − cv − df − investment_rate)`

#### Scenario: Formulário exibe aviso em vez de preço zero
- **WHEN** o usuário informa uma margem que torna o cálculo inviável (`feasible: false`)
- **THEN** o formulário exibe um aviso de margem inviável indicando a margem máxima possível (`max_margin`) no lugar da linha de preço final com `R$ 0,00`

#### Scenario: Margem máxima zerada quando taxas já consomem o preço
- **WHEN** `cv + df + investment_rate >= 1`
- **THEN** `computePrice` retorna `max_margin: 0` e `feasible: false` sem erro, e o formulário comunica que as taxas já consomem o preço

### Requirement: Validação dinâmica do campo de margem

O sistema SHALL restringir dinamicamente o valor máximo aceito no campo de margem de lucro do formulário de produto ao máximo viável `(1 − cv − df − investment_rate)`, em pontos percentuais, em vez de um limite fixo.

#### Scenario: Limite máximo do campo derivado das taxas atuais
- **WHEN** o formulário de produto é renderizado com taxas `cv`, `df` e `investment_rate`
- **THEN** o atributo de valor máximo do campo de margem corresponde a `floor((1 − cv − df − investment_rate) × 100)` pontos percentuais

#### Scenario: Limite acompanha mudança nas taxas
- **WHEN** as taxas de precificação usadas pelo formulário mudam
- **THEN** o valor máximo do campo de margem é recalculado para refletir o novo máximo viável
