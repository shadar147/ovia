import { describe, it, expect, vi } from "vitest";
import { render, screen } from "@testing-library/react";
import { LatencyChart } from "./latency-chart";
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
  computed_at: "2026-02-22T00:00:00Z",
  created_at: "2026-02-22T00:00:00Z",
};

describe("LatencyChart", () => {
  it("renders chart when history has data", () => {
    render(<LatencyChart history={[snapshot]} />);
    expect(screen.getByTestId("echarts-mock")).toBeInTheDocument();
  });

  it("shows empty state when history is empty", () => {
    render(<LatencyChart history={[]} />);
    expect(screen.getByText("Not enough history data yet")).toBeInTheDocument();
    expect(screen.queryByTestId("echarts-mock")).not.toBeInTheDocument();
  });

  it("renders chart even when latency values are null", () => {
    const nullLatency: KpiSnapshot = {
      ...snapshot,
      review_latency_median_hours: null,
      review_latency_p90_hours: null,
    };
    render(<LatencyChart history={[nullLatency]} />);
    expect(screen.getByTestId("echarts-mock")).toBeInTheDocument();
  });
});
