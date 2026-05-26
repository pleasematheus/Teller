## Why

O login com ID numérico causa crash do processo Rust (janela fecha sozinha) porque o comando `login` chama `init_schema` — que executa o schema.sql completo via `execute_batch` — a cada autenticação, e o DuckDB aborta o processo ao encontrar objetos já existentes no banco. Adicionalmente, o lookup de usuário retorna apenas `true`/`false`, desperdiçando o `nickname` já consultado e exibindo uma mensagem genérica em vez de preencher o campo com o apelido encontrado.

## What Changes

- **Mover `init_schema` de `login` para `check_setup`**: o schema completo passa a ser inicializado uma única vez no startup, não a cada login. O comando `login` fica com responsabilidade exclusiva de autenticar.
- **`find_user` retorna `Option<String>` (nickname) em vez de `bool`**: quando o usuário é encontrado, o backend devolve o apelido; quando não encontrado, devolve `null`.
- **Frontend substitui o campo identificador pelo apelido**: ao sair do campo com um ID numérico válido, o valor é trocado pelo apelido correspondente. Se não encontrado, exibe erro no campo.
- **Remover mensagem "Usuário encontrado"**: substituída pelo preenchimento automático do apelido no campo.

## Capabilities

### New Capabilities

_(nenhuma)_

### Modified Capabilities

- `local-auth`: comportamento do lookup leve muda — passa a retornar o `nickname` do usuário encontrado em vez de `bool`, e o campo identificador é substituído pelo apelido na UI.

## Impact

- `src-tauri/src/lib.rs`: comandos `check_setup` e `login` (movimentação do `init_schema`); assinatura e corpo do comando `find_user`
- `src/components/Login.tsx`: handler `handleIdentifierBlur`, estado derivado de `userFound`, JSX do feedback do campo identificador
