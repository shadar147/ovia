import { describe, it, expect } from "vitest";
import { deduplicateHistory, formatWeekLabel } from "./chart-utils";
import type { KpiSnapshot } from "@/lib/api/types";

function makeSnapshot(overrides: Partial<KpiSnapshot> = {}): KpiSnapshot {
  return {
    id: "s1",
    org_id: "org1",
    period_start: "2026-02-03",
    period_end: "2026-02-09",
    delivery_health_score: 75,
    release_risk_score: 42,
    throughput_total: 48,
    throughput_bugs: 5,
    throughput_features: 12,
    throughput_chores: 31,
    review_latency_median_hours: 4.2,
    review_latency_p90_hours: 18.7,
    blocker_count: 0,
    spillover_rate: null,
    cycle_time_p50_hours: null,
    cycle_time_p90_hours: null,
    computed_at: "2026-02-09T00:00:00Z",
    created_at: "2026-02-09T00:00:00Z",
    ...overrides,
  };
}

describe("formatWeekLabel", () => {
  it("formats ISO date to short label", () => {
    expect(formatWeekLabel("2026-02-17", "en")).toBe("Feb 17");
  });
});

describe("deduplicateHistory", () => {
  it("returns empty array for empty input", () => {
    expect(deduplicateHistory([])).toEqual([]);
  });

  it("returns single snapshot unchanged", () => {
    const snap = makeSnapshot();
    const result = deduplicateHistory([snap]);
    expect(result).toHaveLength(1);
    expect(result[0]).toBe(snap);
  });

  it("keeps snapshots with distinct period_start", () => {
    const a = makeSnapshot({ id: "a", period_start: "2026-01-27" });
    const b = makeSnapshot({ id: "b", period_start: "2026-02-03" });
    const result = deduplicateHistory([b, a]);
    expect(result).toHaveLength(2);
    expect(result[0].id).toBe("a");
    expect(result[1].id).toBe("b");
  });

  it("deduplicates by period_start keeping latest computed_at", () => {
    const older = makeSnapshot({
      id: "monthly",
      period_start: "2026-02-01",
      period_end: "2026-02-28",
      computed_at: "2026-02-01T12:00:00Z",
      throughput_total: 30,
    });
    const newer = makeSnapshot({
      id: "weekly",
      period_start: "2026-02-01",
      period_end: "2026-02-07",
      computed_at: "2026-02-08T12:00:00Z",
      throughput_total: 48,
    });
    const result = deduplicateHistory([older, newer]);
    expect(result).toHaveLength(1);
    expect(result[0].id).toBe("weekly");
    expect(result[0].throughput_total).toBe(48);
  });

  it("dedup works regardless of input order", () => {
    const older = makeSnapshot({
      id: "old",
      period_start: "2026-02-01",
      computed_at: "2026-02-01T00:00:00Z",
    });
    const newer = makeSnapshot({
      id: "new",
      period_start: "2026-02-01",
      computed_at: "2026-02-08T00:00:00Z",
    });
    // newer first
    expect(deduplicateHistory([newer, older])[0].id).toBe("new");
    // older first
    expect(deduplicateHistory([older, newer])[0].id).toBe("new");
  });

  it("sorts result by period_start ascending", () => {
    const feb = makeSnapshot({ id: "feb", period_start: "2026-02-10" });
    const jan = makeSnapshot({ id: "jan", period_start: "2026-01-06" });
    const mar = makeSnapshot({ id: "mar", period_start: "2026-03-02" });
    const result = deduplicateHistory([mar, jan, feb]);
    expect(result.map((s) => s.id)).toEqual(["jan", "feb", "mar"]);
  });

  it("handles mixed weekly and monthly snapshots with overlapping dates", () => {
    const weekly1 = makeSnapshot({
      id: "w1",
      period_start: "2026-01-27",
      period_end: "2026-02-02",
      computed_at: "2026-02-02T00:00:00Z",
    });
    const monthly = makeSnapshot({
      id: "m1",
      period_start: "2026-02-01",
      period_end: "2026-02-28",
      computed_at: "2026-02-01T00:00:00Z",
    });
    const weekly2 = makeSnapshot({
      id: "w2",
      period_start: "2026-02-01",
      period_end: "2026-02-07",
      computed_at: "2026-02-08T00:00:00Z",
    });
    const weekly3 = makeSnapshot({
      id: "w3",
      period_start: "2026-02-08",
      period_end: "2026-02-14",
      computed_at: "2026-02-15T00:00:00Z",
    });

    const result = deduplicateHistory([weekly1, monthly, weekly2, weekly3]);
    expect(result).toHaveLength(3);
    // monthly (Feb 1) replaced by weekly2 (Feb 1, newer computed_at)
    expect(result.map((s) => s.id)).toEqual(["w1", "w2", "w3"]);
  });
});
