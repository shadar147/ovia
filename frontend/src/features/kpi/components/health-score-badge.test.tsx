import { describe, it, expect } from "vitest";
import { render, screen } from "@testing-library/react";
import { HealthScoreBadge } from "./health-score-badge";

describe("HealthScoreBadge", () => {
  it("renders Healthy for score >= 80", () => {
    render(<HealthScoreBadge score={85} />);
    expect(screen.getByText("Healthy")).toBeInTheDocument();
  });

  it("renders Healthy for score exactly 80", () => {
    render(<HealthScoreBadge score={80} />);
    expect(screen.getByText("Healthy")).toBeInTheDocument();
  });

  it("renders At Risk for score 60-79", () => {
    render(<HealthScoreBadge score={72} />);
    expect(screen.getByText("At Risk")).toBeInTheDocument();
  });

  it("renders At Risk for score exactly 60", () => {
    render(<HealthScoreBadge score={60} />);
    expect(screen.getByText("At Risk")).toBeInTheDocument();
  });

  it("renders Critical for score < 60", () => {
    render(<HealthScoreBadge score={45} />);
    expect(screen.getByText("Critical")).toBeInTheDocument();
  });

  it("renders N/A for null score", () => {
    render(<HealthScoreBadge score={null} />);
    expect(screen.getByText("N/A")).toBeInTheDocument();
  });

  it("renders Critical for score 0", () => {
    render(<HealthScoreBadge score={0} />);
    expect(screen.getByText("Critical")).toBeInTheDocument();
  });

  it("renders Healthy for score 100", () => {
    render(<HealthScoreBadge score={100} />);
    expect(screen.getByText("Healthy")).toBeInTheDocument();
  });
});
