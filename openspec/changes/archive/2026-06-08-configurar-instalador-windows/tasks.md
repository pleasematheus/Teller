## 1. Metadados do bundle

- [x] 1.1 Adicionar `bundle.publisher` (ex.: "Matheus Fracalossi") e `bundle.copyright` em `src-tauri/tauri.conf.json`
- [x] 1.2 Adicionar `bundle.homepage` e `bundle.shortDescription`/`bundle.longDescription` se aplicavel

## 2. Configuracao NSIS

- [x] 2.1 Adicionar bloco `bundle.windows.nsis` com `installMode: "perUser"`
- [x] 2.2 Definir `nsis.languages: ["PortugueseBR"]`
- [x] 2.3 Habilitar selecao de diretorio (`nsis.allowDowngrades` / `installerIcon` conforme suporte) e atalhos de menu iniciar e desktop
- [x] 2.4 (Opcional) Referenciar arquivo de licenca em `nsis.license` se houver licenca definida

## 3. WebView2 e alvos

- [x] 3.1 Adicionar `bundle.windows.webviewInstallMode: { "type": "downloadBootstrapper" }`
- [x] 3.2 Confirmar `bundle.targets` com `nsis` (primario) e `msi` mantidos

## 4. Validacao de build

- [x] 4.1 Executar `bun run tauri build` no Windows e confirmar geracao do `.exe` NSIS e `.msi`
- [x] 4.2 Verificar que o JSON de config valida contra o schema `tauri.app/config/2` (sem erro de build)

## 5. Testes manuais (testador humano)

- [x] 5.1 MANUAL: instalar o `.exe` NSIS em conta sem admin e confirmar ausencia de prompt UAC
- [x] 5.2 MANUAL: confirmar telas do instalador em portugues
- [x] 5.3 MANUAL: confirmar criacao de atalhos no menu iniciar e desktop, e tela de escolha de diretorio
- [x] 5.4 MANUAL: verificar fabricante/copyright em "Aplicativos e Recursos"
- [x] 5.5 MANUAL: testar instalacao em maquina sem WebView2 e confirmar download/instalacao automatica do runtime

## 6. Documentacao

- [x] 6.1 Documentar comando de build do instalador e caminho do artefato no README ou doc do projeto
