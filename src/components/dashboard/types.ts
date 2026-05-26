export type ModuleId =
  | "estoque"
  | "compras"
  | "producao"
  | "vendas"
  | "produtos"
  | "configuracoes";

export type Tab = {
  id: string;
  module: ModuleId;
};
