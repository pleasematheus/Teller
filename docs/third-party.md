# Recursos de terceiros

O código do Teller usa a licença Apache 2.0. Recursos e dependências de terceiros mantêm suas próprias licenças.

## Ícone Cashier

Os arquivos `src/assets/cashier-svgrepo-com.svg` e `public/cashier-svgrepo-com.svg` correspondem ao ícone [Cashier, do SVG Repo](https://www.svgrepo.com/svg/301787/cashier). A página do recurso identifica a licença como CC0.

## Fonte Inter

O aplicativo distribui a fonte Inter pelo pacote `@fontsource-variable/inter`. O pacote declara a licença SIL Open Font License 1.1. Uma cópia da licença acompanha o aplicativo em `third-party/INTER-LICENSE.txt`.

## Dependências

As dependências declaradas em `bun.lock` e `src-tauri/Cargo.lock` têm licenças próprias. A lista inclui licenças MIT, Apache-2.0, BSD, ISC, MPL-2.0 e outras. Consulte os pacotes para obter os textos e avisos aplicáveis a cada distribuição.

O DuckDB 1.5.3 é compilado junto ao aplicativo. O arquivo `third-party/DUCKDB-1.5.3-LICENSES.txt` reúne a licença do DuckDB e os textos identificados para bibliotecas nativas incluídas no seu manifesto de compilação.

O inventário dos metadados de licença não determina, por si só, quais arquivos e avisos cada instalador deve incluir. A [revisão dos avisos](license-review.md) registra as verificações realizadas e as pendências para a distribuição.
