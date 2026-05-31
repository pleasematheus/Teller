## Purpose

Combobox compartilhado de seleção de produto, usado em venda e compra. Fornece busca insensível a acento por nome e descrição, navegação por teclado, estado vazio explícito e lista flutuante (dialog) ancorada acima do conteúdo.

## Requirements

### Requirement: Combobox de seleção de produto

O sistema SHALL fornecer um componente de combobox compartilhado para selecionar um produto a partir de uma lista, com input de busca ancorado e flutuante (não no fluxo do documento), navegação por teclado e estado vazio explícito. O componente SHALL emitir o produto escolhido via callback `onSelect(product)` e não SHALL conter lógica de carrinho ou de linha de compra.

#### Scenario: Lista ancorada e flutuante
- **WHEN** o usuário abre o combobox
- **THEN** a lista de resultados é renderizada flutuando ancorada ao input, sobre o conteúdo, sem empurrar os elementos abaixo dela

#### Scenario: Navegação por teclado
- **WHEN** o combobox está aberto e o usuário pressiona seta para baixo, seta para cima e Enter
- **THEN** o item destacado muda conforme as setas e o item destacado é selecionado ao pressionar Enter, emitindo `onSelect` com aquele produto

#### Scenario: Fechar com Escape
- **WHEN** o combobox está aberto e o usuário pressiona Escape ou clica fora
- **THEN** a lista fecha sem selecionar nenhum produto

#### Scenario: Estado vazio
- **WHEN** o texto digitado não casa com nenhum produto
- **THEN** a lista exibe uma mensagem de "nenhum produto encontrado" em vez de desaparecer silenciosamente

### Requirement: Busca de produto insensível a acento e ranqueada

O sistema SHALL filtrar produtos por correspondência no `name` e na `description`, ignorando diferenças de acentuação e de caixa, e SHALL ordenar os resultados priorizando correspondências por prefixo sobre correspondências em outras posições. A lógica de busca SHALL ser uma função pura, isolada da camada de UI e coberta por teste.

#### Scenario: Correspondência ignora acento
- **WHEN** o usuário digita "acai" e existe um produto chamado "Açaí 300ml"
- **THEN** "Açaí 300ml" aparece nos resultados

#### Scenario: Correspondência na descrição
- **WHEN** o termo digitado não casa com nenhum `name` mas casa com a `description` de um produto
- **THEN** esse produto aparece nos resultados

#### Scenario: Prefixo ranqueado acima de substring
- **WHEN** o usuário digita "co" e existem os produtos "Coca" (prefixo) e "Chocolate" (substring no meio)
- **THEN** "Coca" aparece antes de "Chocolate" na lista
