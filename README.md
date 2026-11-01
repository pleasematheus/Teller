# Teller

O Teller é um mini ERP desktop para gestão de produtos, compras, estoque, vendas e finanças. O aplicativo armazena os dados localmente em um banco DuckDB.

O projeto está em desenvolvimento. Avalie os fluxos e mantenha cópias de segurança antes de usá-lo com dados reais.

## Funcionalidades

- Cadastro de produtos, componentes e unidades de medida.
- Registro de compras, vendas e movimentações de estoque.
- Configuração de formas de pagamento e acompanhamento financeiro.
- Relatório de resultados com visualização gráfica.
- Cadastro de usuários e configuração da empresa.

## Tecnologias

- Tauri 2 e Rust para o aplicativo desktop e as operações de dados.
- React, TypeScript, Vite e Tailwind CSS para a interface.
- DuckDB para armazenamento local.
- Bun, Vitest e testes Rust para desenvolvimento e verificação.

## Desenvolvimento no Windows

Instale o [Bun](https://bun.sh/), o [Rust](https://www.rust-lang.org/tools/install) e os [pré-requisitos do Tauri para Windows](https://v2.tauri.app/start/prerequisites/), incluindo as ferramentas de compilação C++ e o WebView2.

Na raiz do repositório, instale as dependências e crie a configuração local:

```powershell
bun install --frozen-lockfile
Copy-Item .env.example .env
bun run tauri dev
```

Na primeira execução, o aplicativo cria o banco e solicita o cadastro do primeiro usuário e da empresa. O comando `bun run dev` inicia apenas o servidor da interface; use `bun run tauri dev` para executar os recursos do aplicativo desktop.

### Banco de dados

O arquivo `.env.example` seleciona o ambiente de desenvolvimento e o banco `./teller-dev.db`. Caminhos relativos dependem do diretório de trabalho do processo. Sem `APP_ENV=development`, o aplicativo usa o ambiente de produção e, por padrão, armazena `teller.db` no diretório de dados do aplicativo no Windows. `DB_PATH_DEV` e `DB_PATH_PROD` permitem definir outros caminhos.

O aplicativo cria um backup antes de executar migrações de um banco existente. Faça também backups independentes: feche o Teller por completo, inclusive na bandeja do sistema, antes de copiar o banco. Não inclua bancos, backups ou arquivos `.env` no repositório.

Se a inicialização falhar, consulte o [procedimento de recuperação](docs/database-recovery.md). Preserve o banco e seu WAL antes de qualquer tentativa de recuperação.

## Verificações

```powershell
bun run build
bun run test --run
Set-Location src-tauri
cargo fmt --check
cargo clippy -- -D warnings
cargo test
```

O workflow de CI executa essas verificações no Windows em pushes e pull requests. Para gerar instaladores NSIS e MSI, consulte [docs/build.md](docs/build.md). O workflow de build é manual; a proteção do ambiente `build-approval` depende da configuração do repositório no GitHub.

## Licença

O código do Teller está disponível sob a [Licença Apache 2.0](LICENSE). Consulte as informações sobre [recursos e dependências de terceiros](docs/third-party.md).
