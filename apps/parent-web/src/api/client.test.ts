import { describe, expect, it, vi } from "vitest";
import { ApiClient, ApiError } from "./client";

function authHeader(init?: RequestInit): string | undefined {
  return (init?.headers as Record<string, string> | undefined)?.Authorization;
}

describe("ApiClient", () => {
  it("stores tokens on register and attaches the bearer header", async () => {
    const fetchMock = vi.fn(async (url: string, init?: RequestInit) => {
      if (url.endsWith("/v1/auth/register")) {
        return new Response(JSON.stringify({ accessToken: "acc", refreshToken: "ref" }), { status: 200 });
      }
      if (url.endsWith("/v1/family/children")) {
        expect(authHeader(init)).toBe("Bearer acc");
        return new Response(JSON.stringify({ childId: "child-1" }), { status: 201 });
      }
      throw new Error("unexpected " + url);
    });

    const c = new ApiClient("http://x", fetchMock as unknown as typeof fetch);
    await c.register("mom@example.com", "correct horse");
    expect(c.isAuthenticated).toBe(true);
    expect(await c.createChild()).toBe("child-1");
  });

  it("refreshes the access token once on a 401 and retries", async () => {
    let childCalls = 0;
    const fetchMock = vi.fn(async (url: string, init?: RequestInit) => {
      if (url.endsWith("/v1/auth/register")) {
        return new Response(JSON.stringify({ accessToken: "old", refreshToken: "ref" }), { status: 200 });
      }
      if (url.endsWith("/v1/auth/token")) {
        return new Response(JSON.stringify({ accessToken: "new" }), { status: 200 });
      }
      if (url.endsWith("/v1/family/children")) {
        childCalls++;
        return authHeader(init) === "Bearer old"
          ? new Response(JSON.stringify({ error: "unauthorized" }), { status: 401 })
          : new Response(JSON.stringify({ childId: "child-1" }), { status: 201 });
      }
      throw new Error("unexpected " + url);
    });

    const c = new ApiClient("http://x", fetchMock as unknown as typeof fetch);
    await c.register("mom@example.com", "correct horse");
    expect(await c.createChild()).toBe("child-1");
    expect(childCalls).toBe(2); // first call 401, retried after refresh
  });

  it("does not retry forever when refresh fails", async () => {
    const fetchMock = vi.fn(async (url: string) => {
      if (url.endsWith("/v1/auth/register")) {
        return new Response(JSON.stringify({ accessToken: "old", refreshToken: "ref" }), { status: 200 });
      }
      if (url.endsWith("/v1/auth/token")) {
        return new Response(JSON.stringify({ error: "expired" }), { status: 401 });
      }
      return new Response(JSON.stringify({ error: "unauthorized" }), { status: 401 });
    });
    const c = new ApiClient("http://x", fetchMock as unknown as typeof fetch);
    await c.register("mom@example.com", "correct horse");
    await expect(c.createChild()).rejects.toBeInstanceOf(ApiError);
    expect(c.isAuthenticated).toBe(false); // tokens cleared after failed refresh
  });

  it("throws ApiError with the server message on failure", async () => {
    const fetchMock = vi.fn(async () => new Response(JSON.stringify({ error: "email already registered" }), { status: 409 }));
    const c = new ApiClient("http://x", fetchMock as unknown as typeof fetch);
    await expect(c.register("mom@example.com", "x")).rejects.toMatchObject({
      status: 409,
      message: "email already registered",
    });
  });

  it("requests the long-poll variant when asked", async () => {
    const seen: string[] = [];
    const fetchMock = vi.fn(async (url: string) => {
      seen.push(url);
      if (url.endsWith("/v1/auth/register")) {
        return new Response(JSON.stringify({ accessToken: "acc", refreshToken: "ref" }), { status: 200 });
      }
      return new Response(JSON.stringify([]), { status: 200 });
    });
    const c = new ApiClient("http://x", fetchMock as unknown as typeof fetch);
    await c.register("mom@example.com", "correct horse");
    await c.streamAlerts(true);
    expect(seen.some((u) => u.endsWith("/v1/alerts/stream?wait=1"))).toBe(true);
  });
});
