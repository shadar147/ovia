import { describe, it, expect } from "vitest";
import { kpiKeys } from "./keys";

describe("kpiKeys", () => {
  it("generates latest key", () => {
    expect(kpiKeys.latest()).toEqual(["kpi", "latest"]);
  });

  it("generates history key with default filter", () => {
    expect(kpiKeys.history()).toEqual(["kpi", "history", {}]);
  });

  it("generates history key with filter", () => {
    const key = kpiKeys.history({ limit: 12, period_start: "2026-01-01" });
    expect(key).toEqual(["kpi", "history", { limit: 12, period_start: "2026-01-01" }]);
  });

  it("generates risks key", () => {
    expect(kpiKeys.risks()).toEqual(["kpi", "risks"]);
  });

  it("all keys share common prefix", () => {
    expect(kpiKeys.latest()[0]).toBe("kpi");
    expect(kpiKeys.history()[0]).toBe("kpi");
    expect(kpiKeys.risks()[0]).toBe("kpi");
  });
});
