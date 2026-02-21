import { describe, it, expect } from "vitest";
import { identityKeys } from "./keys";

describe("identityKeys", () => {
  it("generates mapping keys with filter", () => {
    const key = identityKeys.mappings({ status: "conflict", limit: 25 });
    expect(key).toEqual(["identity", "mappings", { status: "conflict", limit: 25 }]);
  });

  it("generates conflict keys", () => {
    const key = identityKeys.conflicts({ sort: "confidence_asc" });
    expect(key).toEqual(["identity", "conflicts", { sort: "confidence_asc" }]);
  });

  it("generates stats key", () => {
    expect(identityKeys.conflictStats()).toEqual(["identity", "conflict-stats"]);
  });
});
