## Purpose

Define onde e como um spinner animado acompanha textos de estado de carregamento na interface (listas, seções, botões e selects), garantindo feedback visual consistente e acessível durante operações assíncronas.

## Requirements

### Requirement: Spinner acompanha texto de carregamento

A interface SHALL exibir um indicador animado (`Spinner`) à esquerda de todo texto que comunica um estado de carregamento ativo (por exemplo "Carregando...", "Salvando...", "Entrando...", "Criando..."). O spinner e o texto MUST estar visíveis simultaneamente enquanto o carregamento estiver em andamento.

#### Scenario: Carregamento de seção ou lista

- **WHEN** uma seção ou lista está buscando dados e exibe o texto "Carregando..."
- **THEN** um spinner animado é renderizado imediatamente à esquerda do texto

#### Scenario: Fim do carregamento de seção

- **WHEN** os dados terminam de carregar
- **THEN** tanto o texto de carregamento quanto o spinner são removidos da tela

### Requirement: Spinner em botões de ação assíncrona

Botões que disparam uma ação assíncrona SHALL exibir um spinner à esquerda do texto de estado enquanto a ação está pendente, mantendo o botão desabilitado durante esse período.

#### Scenario: Botão durante ação em andamento

- **WHEN** o usuário aciona um botão de ação assíncrona (por exemplo "Entrar", "Criar conta", "Salvar e continuar")
- **THEN** o botão fica desabilitado, o texto muda para o estado de progresso (por exemplo "Entrando...") e um spinner aparece à esquerda do texto

#### Scenario: Conclusão da ação do botão

- **WHEN** a ação assíncrona termina
- **THEN** o spinner é removido e o botão retorna ao seu rótulo e estado normais

### Requirement: Spinner em placeholder de seleção

Selects cujas opções dependem de dados ainda em carregamento SHALL indicar o carregamento com spinner adjacente ao controle.

#### Scenario: Select aguardando dados

- **WHEN** um select depende de dados que ainda não chegaram e exibe a opção "Carregando..."
- **THEN** um spinner é exibido adjacente ao select enquanto os dados não chegam

### Requirement: Acessibilidade do indicador

O componente `Spinner` SHALL expor `role="status"` para tecnologias assistivas. Quando o spinner acompanhar um texto que já comunica o mesmo estado, o ícone MUST ser marcado como decorativo (`aria-hidden`) para evitar anúncio redundante.

#### Scenario: Spinner com texto redundante

- **WHEN** o spinner é renderizado ao lado de um texto que já descreve o carregamento
- **THEN** o ícone é marcado como decorativo para leitores de tela e o texto permanece como a fonte do estado anunciado
