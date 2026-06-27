export interface PaymentLineInput {
  amount: number
  is_cash: boolean
}

export interface PaymentCalcResult {
  applied: number[]
  change: number[]
  totalChange: number
  remaining: number
}

export function calcPayments(total: number, lines: PaymentLineInput[]): PaymentCalcResult {
  let remaining = total
  const applied: number[] = []
  const change: number[] = []

  for (const line of lines) {
    if (line.is_cash) {
      const app = Math.min(line.amount, Math.max(0, remaining))
      applied.push(app)
      change.push(line.amount - app)
      remaining -= app
    } else {
      applied.push(line.amount)
      change.push(0)
      remaining -= line.amount
    }
  }

  return {
    applied,
    change,
    totalChange: change.reduce((sum, c) => sum + c, 0),
    remaining,
  }
}
