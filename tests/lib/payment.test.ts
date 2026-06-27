import { describe, it, expect } from "vitest"
import { calcPayments } from "@/lib/payment"

describe("calcPayments", () => {
  // Scenario: Pagamento único em dinheiro com troco
  it("cash payment with change returns applied=total, change=excess", () => {
    const result = calcPayments(48, [{ amount: 50, is_cash: true }])
    expect(result.applied[0]).toBeCloseTo(48)
    expect(result.change[0]).toBeCloseTo(2)
    expect(result.totalChange).toBeCloseTo(2)
    expect(result.remaining).toBeCloseTo(0)
  })

  // Scenario: Recebido igual ao total (troco 0)
  it("cash payment exact amount gives zero change", () => {
    const result = calcPayments(48, [{ amount: 48, is_cash: true }])
    expect(result.applied[0]).toBeCloseTo(48)
    expect(result.change[0]).toBeCloseTo(0)
    expect(result.totalChange).toBeCloseTo(0)
    expect(result.remaining).toBeCloseTo(0)
  })

  // Scenario: Recebido insuficiente
  it("cash payment insufficient amount leaves remaining > 0", () => {
    const result = calcPayments(100, [{ amount: 60, is_cash: true }])
    expect(result.applied[0]).toBeCloseTo(60)
    expect(result.change[0]).toBeCloseTo(0)
    expect(result.remaining).toBeCloseTo(40)
  })

  // Scenario: Split dinheiro + não-dinheiro com troco na linha de dinheiro
  it("split cash and non-cash: non-cash applied first, cash covers rest with change", () => {
    const result = calcPayments(100, [
      { amount: 60, is_cash: false },
      { amount: 50, is_cash: true },
    ])
    expect(result.applied[0]).toBeCloseTo(60)
    expect(result.change[0]).toBeCloseTo(0)
    expect(result.applied[1]).toBeCloseTo(40)
    expect(result.change[1]).toBeCloseTo(10)
    expect(result.totalChange).toBeCloseTo(10)
    expect(result.remaining).toBeCloseTo(0)
  })

  // Scenario: Linha não-dinheiro inalterada
  it("non-cash line is unchanged (no change field)", () => {
    const result = calcPayments(100, [{ amount: 100, is_cash: false }])
    expect(result.applied[0]).toBeCloseTo(100)
    expect(result.change[0]).toBeCloseTo(0)
    expect(result.totalChange).toBeCloseTo(0)
    expect(result.remaining).toBeCloseTo(0)
  })
})
