## Purpose

Navegação intra-módulo via sub-abas para Produtos, Estoque e Vendas. Sub-abas fixas representam views permanentes; sub-abas dinâmicas são abertas ao acionar formulários e permanecem montadas (com estado preservado) até o usuário fechá-las explicitamente ou concluir a ação.

## Requirements

### Requirement: Sub-abas dinâmicas por módulo
Cada módulo (Produtos, Estoque, Vendas) SHALL exibir uma barra de sub-abas no topo do seu conteúdo. Sub-abas fixas representam views permanentes; sub-abas dinâmicas aparecem ao acionar uma ação e somem ao concluir ou ao fechar explicitamente.

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

### Requirement: Preservação de estado ao navegar entre abas
O sistema SHALL preservar o estado do formulário aberto quando o usuário navega para uma sub-aba fixa. O estado só é destruído ao fechar a sub-aba (×) ou concluir a ação (salvar/cancelar).

#### Scenario: Navegação para aba fixa com formulário preenchido
- **WHEN** o usuário clica em uma sub-aba fixa enquanto há campos preenchidos numa sub-aba dinâmica
- **THEN** a sub-aba fixa fica ativa, mas a sub-aba dinâmica permanece visível na barra e seu estado é preservado

#### Scenario: Retorno à sub-aba dinâmica preserva estado
- **WHEN** o usuário clica na sub-aba dinâmica após ter navegado para uma aba fixa
- **THEN** o formulário é exibido com todos os dados preenchidos anteriormente intactos

#### Scenario: Re-abertura da mesma ação preserva estado
- **WHEN** o usuário aciona novamente a mesma ação que já está em andamento (ex: clica "Novo" quando já há "Novo produto" aberto)
- **THEN** o módulo apenas reativa a sub-aba existente sem resetar o formulário

### Requirement: Sub-abas fixas sem controle de fechamento
Sub-abas fixas (Lista, Saldo, Em aberto, Histórico) SHALL ser sempre visíveis e não possuir botão de fechar.

#### Scenario: Click em sub-aba fixa
- **WHEN** o usuário clica em uma sub-aba fixa
- **THEN** a view correspondente é exibida; sub-abas dinâmicas abertas permanecem na barra com estado preservado

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
