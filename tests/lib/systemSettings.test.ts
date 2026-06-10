import { describe, it, expect } from "vitest";
import { parseSystemSettingBool } from "@/lib/systemSettings";

describe("parseSystemSettingBool", () => {
  it("returns true for 'true'", () => {
    expect(parseSystemSettingBool("true")).toBe(true);
  });

  it("returns false for 'false'", () => {
    expect(parseSystemSettingBool("false")).toBe(false);
  });

  it("returns false default for null", () => {
    expect(parseSystemSettingBool(null)).toBe(false);
  });

  it("returns custom default for null", () => {
    expect(parseSystemSettingBool(null, true)).toBe(true);
  });

  it("returns default for undefined", () => {
    expect(parseSystemSettingBool(undefined)).toBe(false);
  });

  it("returns default for invalid value", () => {
    expect(parseSystemSettingBool("yes")).toBe(false);
    expect(parseSystemSettingBool("1")).toBe(false);
    expect(parseSystemSettingBool("")).toBe(false);
  });
});
