## ADDED Requirements

### Requirement: Cálculo dos 6 indicadores do mini-DRE
O sistema SHALL calcular mensalmente os seguintes indicadores em cascata:
- **Receita Bruta** = soma de `Sales_Products.price × quantity` para vendas fechadas no mês + `Financial_Entries` income do mês
- **Receita Líquida** = Receita Bruta − devoluções (`expense_subtype = 'devolucao'`) − descontos (`Sales_Products.price − discounted_price`) − taxas de pagamento (`Sale_Payments.amount_paid − net_amount`)
- **Lucro Bruto** = Receita Líquida − CMV (`Sales_Products.cost_price × quantity` onde `cost_price IS NOT NULL`)
- **Lucro Operacional** = Lucro Bruto − `Financial_Entries` expense com `expense_subtype IN ('despesa', 'custo')`
- **EBIT** = Lucro Operacional − `Financial_Entries` expense com `expense_subtype = 'juros'`
- **Lucro Líquido** = EBIT − `Financial_Entries` expense com `expense_subtype = 'imposto'`

#### Scenario: Mês sem dados retorna zeros
- **WHEN** não há vendas nem lançamentos financeiros no mês selecionado
- **THEN** todos os 6 indicadores são exibidos com valor R$ 0,00

#### Scenario: CMV parcial por falta de cost_price histórico
- **WHEN** existem `Sales_Products` com `cost_price IS NULL` no mês
- **THEN** o CMV é calculado apenas sobre itens com `cost_price NOT NULL` e um aviso "CMV parcial" é exibido

#### Scenario: Resultado negativo (prejuízo)
- **WHEN** qualquer indicador intermediário resulta em valor negativo
- **THEN** o valor é exibido em destaque vermelho na tabela DRE

### Requirement: Tabela DRE mensal com percentuais
O sistema SHALL exibir os 6 indicadores em uma tabela com colunas: nome do indicador, valor em R$ e percentual em relação à Receita Bruta. O seletor de mês SHALL seguir o mesmo padrão do módulo Financeiro (navegação com setas + calendário).

#### Scenario: Percentuais calculados corretamente
- **WHEN** a Receita Bruta é maior que zero
- **THEN** cada linha exibe `valor / Receita Bruta × 100` formatado com uma casa decimal e sufixo `%`

#### Scenario: Receita Bruta zero evita divisão
- **WHEN** a Receita Bruta é zero
- **THEN** a coluna de percentuais exibe `—` em todas as linhas

#### Scenario: Navegação por mês
- **WHEN** o usuário clica na seta de mês anterior ou próximo mês
- **THEN** os 6 indicadores são recalculados para o novo mês selecionado

### Requirement: Sankey Chart do fluxo financeiro
O sistema SHALL exibir um Sankey Chart (ECharts) mostrando o fluxo de receita até o resultado quando todos os indicadores intermediários forem positivos. Os nós SHALL ser:
- **Fontes**: "Receita de Vendas", "Outras Receitas" (se houver `Financial_Entries` income)
- **Intermediários**: "Receita Bruta", "Receita Líquida", "Lucro Bruto", "Lucro Operacional", "EBIT"
- **Folhas (sinks)**: "Devoluções", "Descontos", "Taxas", "CMV", cada categoria de `despesa`/`custo` agrupada por `category`, "Juros", "IR/CSLL", "Lucro Líquido"

#### Scenario: Sankey exibido com lucro positivo
- **WHEN** todos os indicadores intermediários (Receita Líquida, Lucro Bruto, Lucro Operacional, EBIT) são maiores que zero
- **THEN** o Sankey é renderizado mostrando o fluxo completo da Receita Bruta até o Lucro Líquido

#### Scenario: Fallback textual para prejuízo
- **WHEN** qualquer indicador intermediário é menor ou igual a zero
- **THEN** o Sankey não é exibido e a área do gráfico mostra a mensagem "Gráfico indisponível: resultado negativo no período"

#### Scenario: Categorias de despesa como nós individuais
- **WHEN** existem múltiplas categorias distintas de `despesa` ou `custo`
- **THEN** cada categoria aparece como um nó folha separado no Sankey com o valor agregado

### Requirement: Snapshot de CMV capturado ao fechar venda
O sistema SHALL gravar o `cost_price` atual do produto em `Sales_Products.cost_price` no momento em que uma venda é fechada via `close_sale`. Este valor SHALL ser imutável após o fechamento.

#### Scenario: cost_price gravado no fechamento
- **WHEN** o usuário fecha uma venda com itens
- **THEN** cada `Sales_Products` recebe `cost_price = Products.cost_price` vigente naquele momento

#### Scenario: Venda reaberta não altera cost_price
- **WHEN** uma venda fechada é reaberta e fechada novamente (se suportado)
- **THEN** o `cost_price` mantém o valor do primeiro fechamento
