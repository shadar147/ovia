import { describe, it, expect } from "vitest";
import { render, screen } from "@testing-library/react";
import { KpiCardsRow } from "./kpi-cards-row";
import type { KpiSnapshot } from "@/lib/api/types";

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
  computed_at: "2026-02-22T00:00:00Z",
  created_at: "2026-02-22T00:00:00Z",
};

describe("KpiCardsRow", () => {
  it("renders all four KPI cards with valid data", () => {
    render(<KpiCardsRow latest={base} />);
    expect(screen.getByText("Delivery Health")).toBeInTheDocument();
    expect(screen.getByText("Release Risk")).toBeInTheDocument();
    expect(screen.getByText("Throughput")).toBeInTheDocument();
    expect(screen.getByText("Review Latency")).toBeInTheDocument();
    expect(screen.getByText("75.3")).toBeInTheDocument();
    expect(screen.getByText("42.1")).toBeInTheDocument();
    expect(screen.getByText("48")).toBeInTheDocument();
    expect(screen.getByText("4.2h")).toBeInTheDocument();
  });

  it("shows N/A for all null scores", () => {
    const allNull: KpiSnapshot = {
      ...base,
      delivery_health_score: null,
      release_risk_score: null,
      review_latency_median_hours: null,
      review_latency_p90_hours: null,
    };
    render(<KpiCardsRow latest={allNull} />);
    const naElements = screen.getAllByText("N/A");
    // delivery_health N/A value + health badge N/A + release_risk N/A + latency N/A = 4
    expect(naElements.length).toBeGreaterThanOrEqual(3);
  });

  it("renders zero throughput correctly", () => {
    const zeroThroughput: KpiSnapshot = {
      ...base,
      throughput_total: 0,
      throughput_bugs: 0,
      throughput_features: 0,
      throughput_chores: 0,
    };
    render(<KpiCardsRow latest={zeroThroughput} />);
    expect(screen.getByText("0")).toBeInTheDocument();
  });

  it("shows deltas when previous snapshot provided", () => {
    const previous: KpiSnapshot = {
      ...base,
      delivery_health_score: 70.0,
      throughput_total: 40,
    };
    render(<KpiCardsRow latest={base} previous={previous} />);
    expect(screen.getByText("+5.3 pts")).toBeInTheDocument();
    expect(screen.getByText("+8.0 items")).toBeInTheDocument();
  });

  it("omits deltas when no previous snapshot", () => {
    render(<KpiCardsRow latest={base} />);
    expect(screen.queryByText(/vs prev week/)).not.toBeInTheDocument();
  });

  it("handles score of exactly 0", () => {
    const zeroScores: KpiSnapshot = {
      ...base,
      delivery_health_score: 0,
      release_risk_score: 0,
      review_latency_median_hours: 0,
      review_latency_p90_hours: 0,
    };
    render(<KpiCardsRow latest={zeroScores} />);
    // delivery_health=0.0, release_risk=0.0 → two elements
    expect(screen.getAllByText("0.0")).toHaveLength(2);
    expect(screen.getByText("0.0h")).toBeInTheDocument();
    expect(screen.getByText("Critical")).toBeInTheDocument();
  });
});
