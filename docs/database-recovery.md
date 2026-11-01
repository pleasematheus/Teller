# Falha ao iniciar o banco

Se o Teller não puder abrir ou preparar o banco, a tela de inicialização mostra o caminho e o erro. O aplicativo não abre os módulos de gestão.

O Teller não exclui o WAL para tentar abrir um banco. O DuckDB usa esse arquivo para recuperar transações após uma interrupção.

## Recuperação

1. Feche todas as instâncias do Teller, inclusive na bandeja do sistema.
2. Copie o arquivo do banco e seu arquivo `.wal`, se existir, para uma pasta separada. Mantenha também os backups `.bak.*` e os arquivos `.restore` existentes.
3. Investigue o erro na cópia. Não exclua o WAL original.
4. Se houver uma versão compatível do DuckDB que corrija o erro, teste a abertura da cópia com essa versão.
5. Confira vendas, compras, estoque e saldos antes de usar um banco recuperado.

## Migração com falha

O Teller tenta restaurar o backup criado antes da migração. Se houver um WAL pendente, o aplicativo recusa a restauração automática e mantém os arquivos para recuperação. Falhas de cópia ou substituição são exibidas junto com o erro da migração.

O backup é copiado para um arquivo `.restore` antes de substituir o banco. Um arquivo `.restore` já existente não é sobrescrito.

## Verificação manual pendente

- Inicie o aplicativo com uma cópia de banco inválida. Confirme que a tela mostra o caminho e o erro.
- Feche a janela após a falha. Confirme que o aplicativo encerra sem solicitar cadastro.
- Inicie com um banco válido. Confirme que os fluxos de login e cadastro continuam disponíveis.
