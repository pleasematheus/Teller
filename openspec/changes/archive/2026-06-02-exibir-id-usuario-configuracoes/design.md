## Context

`UsuariosSection.tsx` renderiza cada usuário com nome, nickname e status. O campo `id: number` já está no tipo `UserInfo` e é retornado por `list_users` — só não aparece na UI.

## Goals / Non-Goals

**Goals:**
- Exibir `u.id` em cada linha da lista de usuários.

**Non-Goals:**
- Tornar o ID copiável com clique (scope futuro).
- Alterar backend, schema ou commands.
- Exibir ID nos formulários de criação ou alteração de senha.

## Decisions

**Posicionamento do ID**: exibido abaixo do nickname como texto muted (`#<id>`), junto às informações secundárias já existentes. Mantém consistência visual sem introduzir nova coluna ou layout.

## Risks / Trade-offs

Sem riscos. Mudança puramente aditiva e read-only.
