## Why

O Teller declara `nsis` e `msi` como alvos de bundle, mas sem nenhuma customizacao: o instalador sai em ingles, sem metadados de fabricante, sem licenca, sem controle de modo de instalacao e sem garantia de instalacao do WebView2 em maquinas sem internet. Para distribuir o app a usuarios finais (lojistas), o instalador precisa ser em portugues, com identidade da aplicacao e comportamento previsivel.

## What Changes

- Definir metadados Windows no bundle: `publisher`, `copyright`, `homepage` e categoria.
- Configurar o instalador NSIS: idioma `PortugueseBR`, `installMode` por-usuario (sem exigir admin), criacao de atalhos de menu iniciar e desktop, e permitir escolha do diretorio de instalacao.
- Definir estrategia do WebView2 (`downloadBootstrapper`) para instalar a dependencia automaticamente quando ausente.
- Tornar `nsis` o alvo primario de distribuicao no Windows (manter `msi` opcional para ambientes corporativos).
- Documentar o comando de geracao do instalador e onde o artefato e gerado.

## Capabilities

### New Capabilities
- `instalador-windows`: comportamento e configuracao do instalador Windows (NSIS/MSI) do Teller, incluindo idioma, metadados, modo de instalacao, atalhos e dependencia WebView2.

### Modified Capabilities

## Impact

- `src-tauri/tauri.conf.json`: secao `bundle` (metadados, `bundle.windows`, `bundle.windows.nsis`, `bundle.windows.webviewInstallMode`).
- Possivel arquivo de licenca para o instalador (`LICENSE` ou similar referenciado pelo NSIS).
- Sem novas dependencias de codigo Rust/TS. Code signing fica fora de escopo (sem certificado disponivel).
