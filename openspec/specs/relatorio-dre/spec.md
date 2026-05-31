## Purpose

Mini-DRE (Demonstração do Resultado do Exercício) do módulo de Relatórios. Exibe os 6 indicadores financeiros em cascata — Receita Bruta, Receita Líquida, Lucro Bruto, Lucro Operacional, EBIT e Lucro Líquido — para um período customizável, acompanhados de um Sankey Chart que visualiza o fluxo de receita até o resultado.

## Requirements

### Requirement: Cálculo dos 6 indicadores do mini-DRE
O sistema SHALL calcular os seguintes indicadores em cascata para o período selecionado:
- **Receita Bruta** = soma de `Sales_Products.price × quantity` para vendas fechadas no período + `Financial_Entries` income do período
- **Receita Líquida** = Receita Bruta − devoluções (`expense_subtype = 'devolucao'`) − descontos (`Sales_Products.price − discounted_price`) − taxas de pagamento (`Sale_Payments.amount_paid − net_amount`)
- **Lucro Bruto** = Receita Líquida − CMV (`Sales_Products.cost_price × quantity` onde `cost_price IS NOT NULL`)
- **Lucro Operacional** = Lucro Bruto − `Financial_Entries` expense com `expense_subtype IN ('despesa', 'custo')`
- **EBIT** = Lucro Operacional − `Financial_Entries` expense com `expense_subtype = 'juros'`
- **Lucro Líquido** = EBIT − `Financial_Entries` expense com `expense_subtype = 'imposto'`

#### Scenario: Período sem dados retorna zeros
- **WHEN** não há vendas nem lançamentos financeiros no período selecionado
- **THEN** todos os 6 indicadores são exibidos com valor R$ 0,00

#### Scenario: CMV parcial por falta de cost_price histórico
- **WHEN** existem `Sales_Products` com `cost_price IS NULL` no período
- **THEN** o CMV é calculado apenas sobre itens com `cost_price NOT NULL` e um aviso "CMV parcial" é exibido

#### Scenario: Resultado negativo (prejuízo)
- **WHEN** qualquer indicador intermediário resulta em valor negativo
- **THEN** o valor é exibido em destaque vermelho na tabela DRE

### Requirement: Tabela DRE com percentuais e seletor de período
O sistema SHALL exibir os 6 indicadores em uma tabela com colunas: nome do indicador, valor em R$ e percentual em relação à Receita Bruta. O seletor de período SHALL ser um date range picker (Popover + Calendar) que permite escolher data inicial e data final livremente. O período padrão SHALL ser o mês corrente (do primeiro ao último dia).

#### Scenario: Percentuais calculados corretamente
- **WHEN** a Receita Bruta é maior que zero
- **THEN** cada linha exibe `valor / Receita Bruta × 100` formatado com uma casa decimal e sufixo `%`

#### Scenario: Receita Bruta zero evita divisão
- **WHEN** a Receita Bruta é zero
- **THEN** a coluna de percentuais exibe `—` em todas as linhas

#### Scenario: Seleção de período customizado
- **WHEN** o usuário seleciona uma data inicial e uma data final no date range picker
- **THEN** os 6 indicadores são recalculados para o período selecionado

#### Scenario: Período padrão ao abrir o relatório
- **WHEN** o usuário abre o Mini-DRE
- **THEN** o período selecionado é do primeiro ao último dia do mês corrente

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

### Requirement: Snapshot de CMV capturado ao fechar venda
O sistema SHALL gravar o `cost_price` atual do produto em `Sales_Products.cost_price` no momento em que uma venda é fechada via `close_sale`. Este valor SHALL ser imutável após o fechamento.

#### Scenario: cost_price gravado no fechamento
- **WHEN** o usuário fecha uma venda com itens
- **THEN** cada `Sales_Products` recebe `cost_price = Products.cost_price` vigente naquele momento

#### Scenario: Venda reaberta não altera cost_price
- **WHEN** uma venda fechada é reaberta e fechada novamente (se suportado)
- **THEN** o `cost_price` mantém o valor do primeiro fechamento

### Requirement: Navegação entre relatórios
O módulo de Relatórios SHALL exibir uma tela de seleção com cards, um por relatório disponível. O Mini-DRE SHALL ser acessível como um card nessa tela. A view do relatório ativo SHALL exibir um botão de retorno à tela de seleção.

#### Scenario: Tela de seleção exibe cards de relatórios
- **WHEN** o usuário abre o módulo Relatórios
- **THEN** é exibida uma grade de cards com os relatórios disponíveis

#### Scenario: Navegação para o Mini-DRE
- **WHEN** o usuário clica no card "Mini-DRE"
- **THEN** a view do Mini-DRE é exibida com o período padrão (mês corrente)

#### Scenario: Retorno à tela de seleção
- **WHEN** o usuário clica em "Relatórios" / botão voltar dentro do Mini-DRE
- **THEN** a tela de seleção de relatórios é exibida novamente
