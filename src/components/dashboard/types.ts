export type ModuleId =
  | "estoque"
  | "compras"
  | "relatorios"
  | "vendas"
  | "produtos"
  | "configuracoes";

export type Tab = {
  id: string;
  module: ModuleId;
};
