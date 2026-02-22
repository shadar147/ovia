import { describe, it, expect, vi } from "vitest";
import { render, screen } from "@testing-library/react";
import { ThroughputBreakdown } from "./throughput-breakdown";
import type { KpiSnapshot } from "@/lib/api/types";

// Mock echarts-for-react to avoid canvas dependency in tests
vi.mock("echarts-for-react/lib/core", () => ({
  default: (props: Record<string, unknown>) => (
    <div data-testid="echarts-mock" data-option={JSON.stringify(props.option)} />
  ),
}));

const base: KpiSnapshot = {
  id: "s1",
  org_id: "org1",
  period_start: "2026-02-17",
  period_end: "2026-02-23",
  delivery_health_score: 75.3,
  release_risk_score: 42.1,
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
  computed_at: "2026-02-22T00:00:00Z",
  created_at: "2026-02-22T00:00:00Z",
};

describe("ThroughputBreakdown", () => {
  it("renders chart when throughput > 0", () => {
    render(<ThroughputBreakdown snapshot={base} />);
    expect(screen.getByTestId("echarts-mock")).toBeInTheDocument();
  });

  it("shows empty state when throughput_total is 0", () => {
    const zero: KpiSnapshot = {
      ...base,
      throughput_total: 0,
      throughput_bugs: 0,
      throughput_features: 0,
      throughput_chores: 0,
    };
    render(<ThroughputBreakdown snapshot={zero} />);
    expect(screen.getByText("No items delivered this period")).toBeInTheDocument();
    expect(screen.queryByTestId("echarts-mock")).not.toBeInTheDocument();
  });
});
