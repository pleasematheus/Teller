export type EntryType = "income" | "expense";

export type FinancialEntry = {
  id: number;
  entry_type: EntryType;
  name: string;
  category: string;
  amount: number;
  is_fixed: boolean;
  is_recurring: boolean;
  reference_date: string;
};
