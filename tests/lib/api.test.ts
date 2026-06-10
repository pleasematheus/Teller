import { describe, it, expect, vi, beforeEach } from "vitest";

vi.mock("@tauri-apps/api/core", () => ({
  invoke: vi.fn().mockResolvedValue(null),
}));

import { invoke } from "@tauri-apps/api/core";
import { setSessionToken, clearSessionToken, authedInvoke } from "@/lib/api";

const mockInvoke = invoke as ReturnType<typeof vi.fn>;

beforeEach(() => {
  mockInvoke.mockClear();
  clearSessionToken();
});

describe("authedInvoke", () => {
  it("injects sessionToken for protected commands", async () => {
    setSessionToken("tok-abc");
    await authedInvoke("list_products", { page: 1 });
    expect(mockInvoke).toHaveBeenCalledWith("list_products", {
      page: 1,
      sessionToken: "tok-abc",
    });
  });

  it("injects empty string when no token set", async () => {
    await authedInvoke("list_products");
    expect(mockInvoke).toHaveBeenCalledWith("list_products", {
      sessionToken: "",
    });
  });

  it("does not inject sessionToken for public commands", async () => {
    setSessionToken("tok-abc");
    await authedInvoke("login", { identifier: "u", password: "p" });
    expect(mockInvoke).toHaveBeenCalledWith("login", {
      identifier: "u",
      password: "p",
    });
  });

  it("clearSessionToken resets injected token to empty", async () => {
    setSessionToken("tok-abc");
    clearSessionToken();
    await authedInvoke("list_products");
    expect(mockInvoke).toHaveBeenCalledWith("list_products", {
      sessionToken: "",
    });
  });

  it("public commands: check_setup, check_business_setup, find_user, logout", async () => {
    setSessionToken("tok");
    const publicCmds = ["check_setup", "check_business_setup", "find_user", "logout"];
    for (const cmd of publicCmds) {
      mockInvoke.mockClear();
      await authedInvoke(cmd, {});
      expect(mockInvoke).toHaveBeenCalledWith(cmd, {});
    }
  });
});
