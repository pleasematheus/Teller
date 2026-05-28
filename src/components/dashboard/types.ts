export type ModuleId =
  | "estoque"
  | "compras"
  | "relatorios"
  | "vendas"
  | "produtos"
  | "configuracoes"
  | "financeiro";

export type Tab = {
  id: string;
  module: ModuleId;
};
