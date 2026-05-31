## MODIFIED Requirements

### Requirement: Sankey Chart do fluxo financeiro
O sistema SHALL exibir um Sankey Chart (ECharts) mostrando o fluxo de receita até o resultado quando todos os indicadores intermediários forem positivos. Os nós SHALL ser:
- **Intermediários**: "Receita Bruta", "Receita Líquida", "Lucro Bruto", "Lucro Operacional", "EBIT"
- **Folhas (sinks)**: "Deduções", "CMV", cada categoria de `despesa`/`custo` agrupada por `category`, "Juros", "Impostos", "Lucro Líquido"

Todos os valores monetários exibidos no gráfico (tooltip dos fluxos e rótulos dos nós) SHALL ser formatados em Real brasileiro (localidade `pt-BR`, moeda `BRL`), de forma consistente com a tabela do DRE.

#### Scenario: Sankey exibido com resultado positivo
- **WHEN** todos os indicadores intermediários (Receita Líquida, Lucro Bruto, Lucro Operacional, EBIT) são maiores que zero
- **THEN** o Sankey é renderizado mostrando o fluxo completo da Receita Bruta até o Lucro Líquido

#### Scenario: Fallback textual para prejuízo
- **WHEN** qualquer indicador intermediário é menor ou igual a zero
- **THEN** o Sankey não é exibido e a área do gráfico mostra a mensagem "Gráfico indisponível: resultado negativo no período"

#### Scenario: Categorias de despesa como nós individuais
- **WHEN** existem múltiplas categorias distintas de `despesa` ou `custo`
- **THEN** cada categoria aparece como um nó folha separado no Sankey com o valor agregado

#### Scenario: Tooltip do fluxo exibe valor em R$
- **WHEN** o usuário passa o mouse sobre um fluxo (link) ou nó do Sankey
- **THEN** o valor é exibido formatado como moeda em Real (ex.: `R$ 1.000,00`) e não como número cru (ex.: `1000`)

#### Scenario: Rótulo do nó exibe nome e valor em R$
- **WHEN** o Sankey é renderizado
- **THEN** cada nó exibe seu nome acompanhado do valor agregado formatado em R$
