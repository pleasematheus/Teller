## ADDED Requirements

### Requirement: Botão de ajuste visualmente clicável

O botão "Ajuste" na tabela de saldo de estoque SHALL usar `variant="secondary"` para ter aparência de ação clicável, distinguindo-o visualmente das células da tabela.

#### Scenario: Botão Ajuste renderizado na tabela de saldo

- **WHEN** a tabela de saldo de estoque é exibida com ao menos um produto
- **THEN** cada linha exibe o botão "Ajuste" com fundo sólido (`variant="secondary"`), não transparente
