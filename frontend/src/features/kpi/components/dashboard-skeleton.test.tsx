import { describe, it, expect } from "vitest";
import { render, screen } from "@testing-library/react";
import { DashboardSkeleton } from "./dashboard-skeleton";

describe("DashboardSkeleton", () => {
  it("renders four KPI card skeletons", () => {
    const { container } = render(<DashboardSkeleton />);
    // 4 KPI cards + 2 chart cards + 1 health card + 2 bottom cards = 9 Card wrappers
    const cards = container.querySelectorAll("[data-slot='card']");
    expect(cards.length).toBe(9);
  });

  it("renders without crashing", () => {
    const { container } = render(<DashboardSkeleton />);
    expect(container.firstChild).toBeTruthy();
  });
});
