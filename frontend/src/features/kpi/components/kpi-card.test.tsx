import { describe, it, expect } from "vitest";
import { render, screen } from "@testing-library/react";
import { KpiCard } from "./kpi-card";
import { Zap } from "lucide-react";

describe("KpiCard", () => {
  it("renders title and value", () => {
    render(<KpiCard title="Throughput" value="48" icon={Zap} />);
    expect(screen.getByText("Throughput")).toBeInTheDocument();
    expect(screen.getByText("48")).toBeInTheDocument();
  });

  it("renders subtitle when provided", () => {
    render(<KpiCard title="Throughput" value="48" subtitle="items delivered" icon={Zap} />);
    expect(screen.getByText("items delivered")).toBeInTheDocument();
  });

  it("renders positive delta", () => {
    render(<KpiCard title="Throughput" value="48" delta={3} deltaLabel="items" icon={Zap} />);
    expect(screen.getByText("+3.0 items")).toBeInTheDocument();
    expect(screen.getByText("vs prev week")).toBeInTheDocument();
  });

  it("renders negative delta", () => {
    render(<KpiCard title="Latency" value="4.1h" delta={-1.2} deltaLabel="hrs" icon={Zap} />);
    expect(screen.getByText("-1.2 hrs")).toBeInTheDocument();
  });

  it("does not render delta when null", () => {
    render(<KpiCard title="Throughput" value="48" delta={null} icon={Zap} />);
    expect(screen.queryByText("vs prev week")).not.toBeInTheDocument();
  });

  it("renders zero delta with neutral indicator", () => {
    render(<KpiCard title="Throughput" value="48" delta={0} deltaLabel="items" icon={Zap} />);
    expect(screen.getByText("0.0 items")).toBeInTheDocument();
    expect(screen.getByText("vs prev week")).toBeInTheDocument();
  });

  it("renders N/A value correctly", () => {
    render(<KpiCard title="Latency" value="N/A" icon={Zap} />);
    expect(screen.getByText("N/A")).toBeInTheDocument();
  });
});
