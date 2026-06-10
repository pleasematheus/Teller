## Purpose

Infraestrutura de sessão do backend: geração de token criptográfico no login, validação em todos os commands sensíveis e invalidação no logout. Garante que operações de negócio (produtos, estoque, vendas, financeiro, relatórios, configurações) só sejam executadas por usuários autenticados.

## Requirements

### Requirement: Geração de token de sessão no login
O sistema SHALL gerar um token de sessão opaco no backend a cada login bem-sucedido, com no mínimo 32 bytes de aleatoriedade criptográfica do sistema operacional, codificado em hexadecimal, e armazená-lo apenas em memória junto ao `user_id` autenticado.

#### Scenario: Login bem-sucedido gera token
- **WHEN** `login` é chamado com credenciais válidas de usuário ativo
- **THEN** o sistema retorna os dados do usuário e um `session_token` não-vazio, e o estado de sessão do backend passa a conter esse token associado ao `user_id`

#### Scenario: Tokens distintos por login
- **WHEN** dois logins bem-sucedidos ocorrem em sequência
- **THEN** os tokens gerados são diferentes entre si

#### Scenario: Novo login substitui sessão anterior
- **WHEN** um login bem-sucedido ocorre enquanto já existe uma sessão ativa
- **THEN** o token anterior é invalidado e apenas o novo token é aceito

### Requirement: Validação de sessão em commands sensíveis
O sistema SHALL exigir um `session_token` válido em todo command que leia ou modifique dados de negócio (produtos, estoque, vendas, financeiro, relatórios, configurações e settings de usuário), retornando erro genérico de sessão inválida quando o token estiver ausente, vazio ou não corresponder à sessão ativa.

#### Scenario: Token válido permite execução
- **WHEN** um command sensível é chamado com o `session_token` da sessão ativa
- **THEN** o command executa normalmente

#### Scenario: Token inválido é recusado
- **WHEN** um command sensível é chamado com token vazio, inexistente ou de sessão já invalidada
- **THEN** o command retorna `Err("Sessão inválida")` sem executar a operação

#### Scenario: Sem sessão ativa
- **WHEN** um command sensível é chamado antes de qualquer login
- **THEN** o command retorna `Err("Sessão inválida")` sem executar a operação

### Requirement: Commands de bootstrap permanecem públicos
O sistema SHALL manter públicos (sem exigência de sessão) apenas os commands necessários antes da autenticação: `check_setup`, `check_business_setup`, `find_user` e `login`. Os commands `create_user` e `create_business_settings` SHALL ser públicos apenas enquanto seu estado de bootstrap não estiver concluído (nenhum usuário ativo / nenhum registro em `Business_Settings`, respectivamente) e exigir sessão válida depois disso.

#### Scenario: Setup inicial sem sessão
- **WHEN** `create_user` é chamado e não existe nenhum usuário ativo no banco
- **THEN** o usuário é criado sem exigência de token

#### Scenario: Criação de usuário pós-setup exige sessão
- **WHEN** `create_user` é chamado já existindo ao menos um usuário ativo e o token é inválido ou ausente
- **THEN** o command retorna `Err("Sessão inválida")` sem criar o usuário

#### Scenario: Configuração da empresa pós-setup exige sessão
- **WHEN** `create_business_settings` é chamado já existindo registro em `Business_Settings` e o token é inválido ou ausente
- **THEN** o command retorna `Err("Sessão inválida")` sem persistir

### Requirement: Logout invalida a sessão
O sistema SHALL expor um command `logout` que remove a sessão ativa do estado em memória, tornando o token anterior inválido para chamadas subsequentes.

#### Scenario: Logout invalida token
- **WHEN** `logout` é chamado com o token da sessão ativa
- **THEN** chamadas subsequentes a commands sensíveis com esse token retornam `Err("Sessão inválida")`

#### Scenario: Logout sem sessão ativa é inofensivo
- **WHEN** `logout` é chamado sem nenhuma sessão ativa
- **THEN** o command retorna sucesso sem efeito colateral

### Requirement: Token de sessão não persiste
O sistema SHALL manter o token de sessão exclusivamente em memória, tanto no backend (estado gerenciado do Tauri) quanto no frontend (estado React), nunca gravando-o em banco, arquivo, `localStorage` ou `sessionStorage`.

#### Scenario: Sessão morre com o processo
- **WHEN** o app é encerrado e reaberto
- **THEN** nenhuma sessão anterior é restaurada e um novo login é exigido

#### Scenario: Frontend não persiste token
- **WHEN** o frontend recebe o `session_token` do login
- **THEN** o token é mantido apenas em estado de memória do React, sem escrita em storage persistente
