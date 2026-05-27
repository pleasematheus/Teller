import { describe, it, expect } from "vitest";
import { resolveTheme } from "./theme";

describe("resolveTheme", () => {
  it("returns light when theme is light", () => {
    expect(resolveTheme("light", false)).toBe("light");
    expect(resolveTheme("light", true)).toBe("light");
  });

  it("returns dark when theme is dark", () => {
    expect(resolveTheme("dark", false)).toBe("dark");
    expect(resolveTheme("dark", true)).toBe("dark");
  });

  it("returns dark when theme is system and SO is dark", () => {
    expect(resolveTheme("system", true)).toBe("dark");
  });

  it("returns light when theme is system and SO is light", () => {
    expect(resolveTheme("system", false)).toBe("light");
  });
});
