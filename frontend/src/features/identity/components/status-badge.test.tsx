import { describe, it, expect } from "vitest";
import { render, screen } from "@testing-library/react";
import { StatusBadge, ConfidenceBadge } from "./status-badge";

describe("StatusBadge", () => {
  it("renders auto status", () => {
    render(<StatusBadge status="auto" />);
    expect(screen.getByText("Auto")).toBeInTheDocument();
  });

  it("renders conflict status", () => {
    render(<StatusBadge status="conflict" />);
    expect(screen.getByText("Conflict")).toBeInTheDocument();
  });

  it("renders verified status", () => {
    render(<StatusBadge status="verified" />);
    expect(screen.getByText("Verified")).toBeInTheDocument();
  });
});

describe("ConfidenceBadge", () => {
  it("renders high confidence", () => {
    render(<ConfidenceBadge confidence={0.92} />);
    expect(screen.getByText("92%")).toBeInTheDocument();
  });

  it("renders low confidence", () => {
    render(<ConfidenceBadge confidence={0.35} />);
    expect(screen.getByText("35%")).toBeInTheDocument();
  });
});
