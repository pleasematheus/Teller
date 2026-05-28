## ADDED Requirements

### Requirement: Sub-abas dinâmicas por módulo
Cada módulo (Produtos, Estoque, Vendas) SHALL exibir uma barra de sub-abas no topo do seu conteúdo. Sub-abas fixas representam views permanentes; sub-abas dinâmicas aparecem ao acionar uma ação e somem ao concluir ou cancelar.

#### Scenario: Sub-aba dinâmica aparece ao acionar ação
- **WHEN** o usuário clica em "Novo", "Editar", "Ajuste" ou "Fechar pedido"
- **THEN** uma nova sub-aba com label descritivo aparece na barra e fica ativa, exibindo o formulário correspondente em largura total

#### Scenario: Sub-aba dinâmica some ao salvar
- **WHEN** o usuário salva ou confirma com sucesso o formulário numa sub-aba dinâmica
- **THEN** a sub-aba é removida e o módulo navega de volta para a sub-aba fixa padrão

#### Scenario: Sub-aba dinâmica some ao cancelar via botão
- **WHEN** o usuário clica em "Cancelar" dentro de um formulário numa sub-aba dinâmica
- **THEN** a sub-aba é removida e o módulo navega de volta para a sub-aba fixa padrão, sem persistir alterações

#### Scenario: Sub-aba dinâmica some ao fechar pelo ×
- **WHEN** o usuário clica no botão × da sub-aba dinâmica
- **THEN** a sub-aba é removida e o módulo navega de volta para a sub-aba fixa padrão, sem persistir alterações

### Requirement: Perda silenciosa de estado não salvo
O sistema SHALL descartar silenciosamente qualquer estado não salvo de um formulário ao remover sua sub-aba dinâmica.

#### Scenario: Navegação para aba fixa com formulário preenchido
- **WHEN** o usuário clica em uma sub-aba fixa enquanto há campos preenchidos numa sub-aba dinâmica
- **THEN** a sub-aba dinâmica é removida, seu estado é descartado sem aviso, e a sub-aba fixa fica ativa

### Requirement: Sub-abas fixas sem controle de fechamento
Sub-abas fixas (Lista, Saldo, Em aberto, Histórico) SHALL ser sempre visíveis e não possuir botão de fechar.

#### Scenario: Click em sub-aba fixa
- **WHEN** o usuário clica em uma sub-aba fixa
- **THEN** a view correspondente é exibida e qualquer sub-aba dinâmica aberta é descartada

### Requirement: Formulários ocupam largura total do módulo
Formulários exibidos como sub-abas SHALL ocupar a área de conteúdo completa do módulo (sem sobreposição), com largura máxima `max-w-lg` centralizada horizontalmente.

#### Scenario: Abrir formulário de produto
- **WHEN** a sub-aba "Novo produto" ou "Editar: [nome]" está ativa
- **THEN** o formulário ocupa toda a área de conteúdo do módulo Produtos, sem Sheet overlay

#### Scenario: Abrir formulário de fechar pedido
- **WHEN** a sub-aba "Fechar #[id]" está ativa
- **THEN** o formulário ocupa toda a área de conteúdo do módulo Vendas, sem Sheet overlay

### Requirement: Labels das sub-abas dinâmicas
O sistema SHALL usar labels específicos por contexto nas sub-abas dinâmicas.

#### Scenario: Labels no módulo Produtos
- **WHEN** uma sub-aba dinâmica de produto está ativa
- **THEN** o label é "Novo produto" para criação, ou "Editar: [nome do produto]" para edição

#### Scenario: Labels no módulo Estoque
- **WHEN** uma sub-aba dinâmica de ajuste está ativa
- **THEN** o label é "Ajuste"

#### Scenario: Labels no módulo Vendas
- **WHEN** uma sub-aba dinâmica de venda está ativa
- **THEN** o label é "Novo pedido" para criação, "Pedido #[id]" para visualização, ou "Fechar #[id]" para fechamento
