## ADDED Requirements

### Requirement: Seleção de ambiente via APP_ENV

O sistema SHALL determinar o ambiente de execução a partir da variável de ambiente `APP_ENV`, aceitando os valores `development` e `production`. Qualquer valor ausente ou não reconhecido SHALL ser tratado como `production`.

#### Scenario: APP_ENV development

- **WHEN** `APP_ENV` é `development`
- **THEN** o sistema resolve o caminho do banco pelo ramo de desenvolvimento

#### Scenario: APP_ENV production

- **WHEN** `APP_ENV` é `production`
- **THEN** o sistema resolve o caminho do banco pelo ramo de produção

#### Scenario: APP_ENV ausente cai em produção

- **WHEN** `APP_ENV` não está definida
- **THEN** o sistema resolve o caminho do banco pelo ramo de produção

#### Scenario: APP_ENV desconhecido cai em produção

- **WHEN** `APP_ENV` tem um valor não reconhecido (exemplo `staging`)
- **THEN** o sistema resolve o caminho do banco pelo ramo de produção

### Requirement: Resolução do caminho em desenvolvimento

No ramo de desenvolvimento, o sistema SHALL usar o caminho definido em `DB_PATH_DEV`. Quando `DB_PATH_DEV` estiver ausente ou vazio, o sistema SHALL usar o caminho padrão `./teller-dev.db`.

#### Scenario: DB_PATH_DEV definido

- **WHEN** `APP_ENV` é `development` e `DB_PATH_DEV` é `./custom-dev.db`
- **THEN** o caminho do banco resolvido é `./custom-dev.db`

#### Scenario: DB_PATH_DEV ausente usa default

- **WHEN** `APP_ENV` é `development` e `DB_PATH_DEV` está ausente ou vazio
- **THEN** o caminho do banco resolvido é `./teller-dev.db`

### Requirement: Resolução do caminho em produção

No ramo de produção, o sistema SHALL usar o caminho definido em `DB_PATH_PROD` quando este for não vazio. Quando `DB_PATH_PROD` estiver ausente ou vazio, o sistema SHALL usar `teller.db` dentro do diretório de dados da aplicação (`app_data_dir`).

#### Scenario: DB_PATH_PROD definido

- **WHEN** `APP_ENV` é `production` e `DB_PATH_PROD` é `D:/dados/teller.db`
- **THEN** o caminho do banco resolvido é `D:/dados/teller.db`

#### Scenario: DB_PATH_PROD vazio usa app_data_dir

- **WHEN** `APP_ENV` é `production` e `DB_PATH_PROD` está ausente ou vazio
- **THEN** o caminho do banco resolvido é `teller.db` dentro de `app_data_dir`

### Requirement: Remoção da variável DB_PATH

O sistema SHALL deixar de ler a variável `DB_PATH`. A presença de `DB_PATH` no ambiente SHALL não ter efeito sobre a resolução do caminho do banco.

#### Scenario: DB_PATH legado é ignorado

- **WHEN** `DB_PATH` está definida mas `APP_ENV` não
- **THEN** o sistema ignora `DB_PATH` e resolve pelo ramo de produção (fallback `app_data_dir/teller.db`)
