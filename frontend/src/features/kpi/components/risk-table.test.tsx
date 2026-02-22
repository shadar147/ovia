import { describe, it, expect } from "vitest";
import { render, screen } from "@testing-library/react";
import userEvent from "@testing-library/user-event";
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

  it("renders pipeline entity type", () => {
    const pipeline = { ...mockRisk, id: "r3", entity_type: "pipeline", title: "Build failed" };
    render(<RiskTable risks={[pipeline]} />);
    expect(screen.getByText("Pipeline")).toBeInTheDocument();
  });

  it("handles unknown entity type gracefully", () => {
    const unknown = { ...mockRisk, id: "r4", entity_type: "deployment", title: "Deploy issue" };
    render(<RiskTable risks={[unknown]} />);
    expect(screen.getByText("Deployment")).toBeInTheDocument();
  });

  it("renders zero age days", () => {
    const fresh = { ...mockRisk, age_days: 0 };
    render(<RiskTable risks={[fresh]} />);
    expect(screen.getByText("0d")).toBeInTheDocument();
  });

  it("renders high age with red styling", () => {
    const old = { ...mockRisk, age_days: 30 };
    render(<RiskTable risks={[old]} />);
    expect(screen.getByText("30d")).toBeInTheDocument();
  });

  it("handles status with underscores", () => {
    const underscoreStatus = { ...mockRisk, status: "in_review" };
    render(<RiskTable risks={[underscoreStatus]} />);
    expect(screen.getByText("in review")).toBeInTheDocument();
  });

  it("does not show pagination for <= 20 items", () => {
    const risks = Array.from({ length: 20 }, (_, i) => ({
      ...mockRisk,
      id: `r-${i}`,
    }));
    render(<RiskTable risks={risks} />);
    expect(screen.queryByText(/of 20/)).not.toBeInTheDocument();
  });

  it("shows pagination for > 20 items", () => {
    const risks = Array.from({ length: 25 }, (_, i) => ({
      ...mockRisk,
      id: `r-${i}`,
      title: `Risk ${i}`,
    }));
    render(<RiskTable risks={risks} />);
    expect(screen.getByText("1–20 of 25")).toBeInTheDocument();
  });

  it("navigates pages with next/prev buttons", async () => {
    const user = userEvent.setup();
    const risks = Array.from({ length: 25 }, (_, i) => ({
      ...mockRisk,
      id: `r-${i}`,
      title: `Risk ${i}`,
    }));
    render(<RiskTable risks={risks} />);

    // Page 1: shows 1-20
    expect(screen.getByText("1–20 of 25")).toBeInTheDocument();
    expect(screen.getByText("Risk 0")).toBeInTheDocument();

    // Go to page 2
    const buttons = screen.getAllByRole("button");
    const nextBtn = buttons[buttons.length - 1];
    await user.click(nextBtn);

    expect(screen.getByText("21–25 of 25")).toBeInTheDocument();
    expect(screen.getByText("Risk 20")).toBeInTheDocument();
    expect(screen.queryByText("Risk 0")).not.toBeInTheDocument();
  });
});
