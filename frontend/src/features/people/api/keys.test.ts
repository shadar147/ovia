import { describe, it, expect } from "vitest";
import { peopleKeys } from "./keys";

describe("peopleKeys", () => {
  it("generates list keys with filter", () => {
    const key = peopleKeys.list({ search: "alice", status: "active", limit: 25 });
    expect(key).toEqual(["people", "list", { search: "alice", status: "active", limit: 25 }]);
  });

  it("generates list keys with empty filter", () => {
    expect(peopleKeys.list()).toEqual(["people", "list", {}]);
  });

  it("generates detail key", () => {
    expect(peopleKeys.detail("abc-123")).toEqual(["people", "detail", "abc-123"]);
  });
});
