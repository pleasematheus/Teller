# Build

## Instalador Windows

```bash
bun run tauri build
```

Gera dois instaladores em `src-tauri/target/release/bundle/`:

| Alvo | Caminho | Uso recomendado |
|------|---------|-----------------|
| NSIS | `nsis/Teller_<versao>_x64-setup.exe` | Distribuicao para usuarios finais (sem admin) |
| MSI  | `msi/Teller_<versao>_x64_en-US.msi`  | Deploy corporativo via GPO |

### Configuracao

- Idioma: Portugues do Brasil (PortugueseBR)
- Modo de instalacao: por usuario (`currentUser`, instala em `%LOCALAPPDATA%`, sem UAC)
- WebView2: baixado automaticamente se ausente (`downloadBootstrapper`)
- Atalhos criados: menu iniciar e area de trabalho

### Aviso SmartScreen

O instalador nao possui assinatura de codigo (Authenticode). O Windows exibira aviso "editor desconhecido" na primeira execucao. Isso e esperado.
