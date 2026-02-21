import { describe, it, expect, vi, beforeEach } from "vitest";
import { api, ApiError } from "./http";

// Mock env
vi.mock("@/lib/env", () => ({
  env: {
    NEXT_PUBLIC_API_URL: "http://localhost:8080",
    NEXT_PUBLIC_ORG_ID: "test-org-id",
  },
}));

describe("api http client", () => {
  beforeEach(() => {
    vi.restoreAllMocks();
  });

  it("makes GET requests with org-id header", async () => {
    const mockData = [{ id: "1" }];
    global.fetch = vi.fn().mockResolvedValue({
      ok: true,
      status: 200,
      json: () => Promise.resolve(mockData),
    });

    const result = await api("/team/identity-mappings");
    expect(result).toEqual(mockData);
    expect(global.fetch).toHaveBeenCalledWith(
      "http://localhost:8080/team/identity-mappings",
      expect.objectContaining({
        headers: expect.objectContaining({
          "x-org-id": "test-org-id",
        }),
      }),
    );
  });

  it("makes POST requests with JSON body", async () => {
    global.fetch = vi.fn().mockResolvedValue({
      ok: true,
      status: 204,
      json: () => Promise.reject(new Error("no body")),
    });

    const result = await api("/team/identity-mappings/confirm", {
      method: "POST",
      body: { link_id: "abc", verified_by: "admin" },
    });
    expect(result).toBeUndefined();
    expect(global.fetch).toHaveBeenCalledWith(
      "http://localhost:8080/team/identity-mappings/confirm",
      expect.objectContaining({
        method: "POST",
        headers: expect.objectContaining({
          "content-type": "application/json",
          "x-org-id": "test-org-id",
        }),
        body: '{"link_id":"abc","verified_by":"admin"}',
      }),
    );
  });

  it("throws ApiError on non-ok response", async () => {
    global.fetch = vi.fn().mockResolvedValue({
      ok: false,
      status: 404,
      statusText: "Not Found",
      json: () => Promise.resolve({ error: "not found" }),
    });

    await expect(api("/missing")).rejects.toThrow(ApiError);
    await expect(api("/missing")).rejects.toMatchObject({
      status: 404,
      statusText: "Not Found",
    });
  });
});
