import { describe, it, expect } from "vitest";
import { render, screen } from "@testing-library/react";
import { RiskTable } from "./risk-table";
import type { RiskItem } from "@/lib/api/types";

const mockRisk: RiskItem = {
  id: "r1",
  org_id: "org1",
  snapshot_id: "s1",
  entity_type: "pull_request",
  title: "Stale PR: Refactor auth middleware",
  owner: "alice.chen",
  age_days: 12,
  impact_scope: "auth-service",
  status: "open",
  source_url: "https://gitlab.example.com/mr/187",
  created_at: "2026-02-10T00:00:00Z",
};

describe("RiskTable", () => {
  it("renders risk rows", () => {
    render(<RiskTable risks={[mockRisk]} />);
    expect(screen.getByText("Pull Request")).toBeInTheDocument();
    expect(screen.getByText("Stale PR: Refactor auth middleware")).toBeInTheDocument();
    expect(screen.getByText("alice.chen")).toBeInTheDocument();
    expect(screen.getByText("12d")).toBeInTheDocument();
    expect(screen.getByText("open")).toBeInTheDocument();
  });

  it("shows Unassigned when owner is null", () => {
    const noOwner = { ...mockRisk, owner: null };
    render(<RiskTable risks={[noOwner]} />);
    expect(screen.getByText("Unassigned")).toBeInTheDocument();
  });

  it("hides external link when source_url is null", () => {
    const noUrl = { ...mockRisk, source_url: null };
    render(<RiskTable risks={[noUrl]} />);
    expect(screen.queryByRole("link")).not.toBeInTheDocument();
  });

  it("renders external link when source_url is present", () => {
    render(<RiskTable risks={[mockRisk]} />);
    const link = screen.getByRole("link");
    expect(link).toHaveAttribute("href", "https://gitlab.example.com/mr/187");
    expect(link).toHaveAttribute("target", "_blank");
  });

  it("shows empty message when no risks", () => {
    render(<RiskTable risks={[]} />);
    expect(screen.getByText(/No risks detected/)).toBeInTheDocument();
  });

  it("renders multiple rows", () => {
    const risks = [
      mockRisk,
      { ...mockRisk, id: "r2", entity_type: "issue", title: "Blocker: DB migration", status: "blocked" },
    ];
    render(<RiskTable risks={risks} />);
    expect(screen.getByText("Pull Request")).toBeInTheDocument();
    expect(screen.getByText("Issue")).toBeInTheDocument();
  });
});
