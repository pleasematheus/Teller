# Revisão de avisos de terceiros

## Escopo

Esta revisão usa os arquivos `bun.lock` e `src-tauri/Cargo.lock` atuais. A análise da interface usa um build separado com mapas de código. A análise Rust usa a árvore de dependências normais para Windows x64. Essas listas incluem dependências transitivas e podem incluir componentes que não chegam ao executável final.

## Recursos e avisos preparados

- Os dois arquivos do ícone Cashier correspondem ao [arquivo do SVG Repo](https://www.svgrepo.com/svg/301787/cashier), cuja página informa CC0.
- A licença OFL 1.1 da fonte Inter está em `public/third-party/INTER-LICENSE.txt`.
- O arquivo `public/third-party/DUCKDB-1.5.3-LICENSES.txt` reúne textos do [DuckDB v1.5.3](https://github.com/duckdb/duckdb/tree/v1.5.3) e das bibliotecas nativas identificadas no manifesto de compilação do pacote.

## Pendências

- A análise do mapa de código identificou 62 instâncias de pacotes JavaScript. A árvore Rust identificou 274 pacotes de dependência. Os textos encontrados estão no [rascunho de avisos](third-party-notices-draft.txt). Ele não está incluído no build e não é uma lista definitiva para distribuição.
- Os textos ausentes dos pacotes Radix foram obtidos no commit identificado pelo registro npm. Os textos dos crates Rust ausentes foram obtidos nos commits identificados por seus metadados de publicação. Para `selectors`, o rascunho inclui a MPL 2.0 e a [origem do código](https://github.com/servo/stylo/tree/635e1a19d02960588a00e189bd4bd5bdb150ec3d/selectors). Confira as condições aplicáveis à distribuição dos arquivos sob MPL.
- Dois pacotes JavaScript permanecem sem texto completo confirmado para a versão exata: `@hugeicons/core-free-icons@4.1.4` e `react-remove-scroll-bar@2.3.8`. Ambos declaram MIT em seus metadados. O primeiro lista um arquivo LICENSE ausente do pacote; o segundo informa MIT no README, mas não inclui o texto completo.
- O build separado não é uma inspeção dos instaladores de release. Gere novos instaladores após concluir os avisos e confira se eles contêm os textos necessários.

O inventário não comprova a conformidade completa da distribuição. Preserve os avisos e textos de licença de cada componente distribuído.
