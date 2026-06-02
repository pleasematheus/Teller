## Why

Administradores precisam referenciar usuários por ID ao configurar integrações ou depurar logs, mas o ID nunca é exibido na tela. O campo já existe no tipo `UserInfo` e é retornado pelo command `list_users` — apenas não renderizado.

## What Changes

- O ID numérico de cada usuário passa a ser exibido na listagem da seção Usuários em Configurações.
- Nenhuma alteração de backend, schema ou command necessária.

## Capabilities

### New Capabilities

Nenhuma.

### Modified Capabilities

- `configuracoes-usuarios`: A listagem de usuários passa a exibir o ID de cada registro.

## Impact

- `src/components/configuracoes/UsuariosSection.tsx`: adicionar render do `u.id` no item de lista.
- `openspec/specs/configuracoes-usuarios/spec.md`: atualizar requisito de listagem para incluir ID.
