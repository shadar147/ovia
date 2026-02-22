import { describe, it, expect, vi } from "vitest";
import { render, screen } from "@testing-library/react";
import { ThroughputChart } from "./throughput-chart";
import type { KpiSnapshot } from "@/lib/api/types";

vi.mock("echarts-for-react/lib/core", () => ({
  default: (props: Record<string, unknown>) => (
    <div data-testid="echarts-mock" data-option={JSON.stringify(props.option)} />
  ),
}));

const snapshot: KpiSnapshot = {
  id: "s1",
  org_id: "org1",
  period_start: "2026-02-17",
  period_end: "2026-02-23",
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
  computed_at: "2026-02-22T00:00:00Z",
  created_at: "2026-02-22T00:00:00Z",
};

describe("ThroughputChart", () => {
  it("renders chart when history has data", () => {
    render(<ThroughputChart history={[snapshot]} />);
    expect(screen.getByTestId("echarts-mock")).toBeInTheDocument();
  });

  it("shows empty state when history is empty", () => {
    render(<ThroughputChart history={[]} />);
    expect(screen.getByText("Not enough history data yet")).toBeInTheDocument();
    expect(screen.queryByTestId("echarts-mock")).not.toBeInTheDocument();
  });

  it("renders correctly with pre-deduplicated history (no duplicate dates)", () => {
    const week1: KpiSnapshot = {
      ...snapshot,
      id: "w1",
      period_start: "2026-02-03",
      period_end: "2026-02-09",
    };
    const week2: KpiSnapshot = {
      ...snapshot,
      id: "w2",
      period_start: "2026-02-10",
      period_end: "2026-02-16",
    };
    render(<ThroughputChart history={[week2, week1]} />);
    const el = screen.getByTestId("echarts-mock");
    const option = JSON.parse(el.getAttribute("data-option")!);
    // X axis labels should be sorted and unique
    expect(option.xAxis.data).toHaveLength(2);
    expect(option.xAxis.data[0]).toBe("Feb 3");
    expect(option.xAxis.data[1]).toBe("Feb 10");
  });
});
