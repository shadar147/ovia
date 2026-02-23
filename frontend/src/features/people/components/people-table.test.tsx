import { describe, it, expect, vi } from "vitest";
import { render, screen } from "@testing-library/react";
import userEvent from "@testing-library/user-event";
import { PeopleTable } from "./people-table";
import type { PersonResponse } from "@/lib/api/types";

const mockPerson: PersonResponse = {
  id: "p1",
  display_name: "Alice Chen",
  primary_email: "alice@example.com",
  avatar_url: null,
  team: "Platform",
  role: "Engineer",
  status: "active",
  identity_count: 3,
  created_at: "2026-01-15T00:00:00Z",
  updated_at: "2026-02-20T00:00:00Z",
};

describe("PeopleTable", () => {
  it("renders person rows with name, team, role, identities", () => {
    render(
      <PeopleTable
        data={[mockPerson]}
        total={1}
        page={0}
        pageSize={25}
        onPageChange={() => {}}
      />,
    );
    expect(screen.getByText("Alice Chen")).toBeInTheDocument();
    expect(screen.getByText("alice@example.com")).toBeInTheDocument();
    expect(screen.getByText("Platform")).toBeInTheDocument();
    expect(screen.getByText("Engineer")).toBeInTheDocument();
    expect(screen.getByText("3")).toBeInTheDocument();
    expect(screen.getByText("Active")).toBeInTheDocument();
  });

  it("shows dash for missing team, role, and zero identities", () => {
    const noTeam: PersonResponse = {
      ...mockPerson,
      id: "p2",
      team: null,
      role: null,
      identity_count: 0,
    };
    render(
      <PeopleTable
        data={[noTeam]}
        total={1}
        page={0}
        pageSize={25}
        onPageChange={() => {}}
      />,
    );
    const dashes = screen.getAllByText("—");
    expect(dashes.length).toBeGreaterThanOrEqual(2);
  });

  it("renders inactive status badge", () => {
    const inactive: PersonResponse = { ...mockPerson, status: "inactive" };
    render(
      <PeopleTable
        data={[inactive]}
        total={1}
        page={0}
        pageSize={25}
        onPageChange={() => {}}
      />,
    );
    expect(screen.getByText("Inactive")).toBeInTheDocument();
  });

  it("shows no-results message when data is empty", () => {
    render(
      <PeopleTable
        data={[]}
        total={0}
        page={0}
        pageSize={25}
        onPageChange={() => {}}
      />,
    );
    expect(screen.getByText(/No people match/)).toBeInTheDocument();
  });

  it("renders page info with from-to of total", () => {
    const people = Array.from({ length: 3 }, (_, i) => ({
      ...mockPerson,
      id: `p-${i}`,
      display_name: `Person ${i}`,
    }));
    render(
      <PeopleTable
        data={people}
        total={50}
        page={0}
        pageSize={25}
        onPageChange={() => {}}
      />,
    );
    expect(screen.getByText("1–25 of 50")).toBeInTheDocument();
  });

  it("disables previous button on first page", () => {
    render(
      <PeopleTable
        data={[mockPerson]}
        total={50}
        page={0}
        pageSize={25}
        onPageChange={() => {}}
      />,
    );
    const prevBtn = screen.getByText("Previous");
    expect(prevBtn).toBeDisabled();
  });

  it("disables next button on last page", () => {
    render(
      <PeopleTable
        data={[mockPerson]}
        total={25}
        page={0}
        pageSize={25}
        onPageChange={() => {}}
      />,
    );
    const nextBtn = screen.getByText("Next");
    expect(nextBtn).toBeDisabled();
  });

  it("calls onPageChange when clicking next", async () => {
    const user = userEvent.setup();
    const onPageChange = vi.fn();
    render(
      <PeopleTable
        data={[mockPerson]}
        total={50}
        page={0}
        pageSize={25}
        onPageChange={onPageChange}
      />,
    );

    await user.click(screen.getByText("Next"));
    expect(onPageChange).toHaveBeenCalledWith(1);
  });

  it("calls onPageChange when clicking previous", async () => {
    const user = userEvent.setup();
    const onPageChange = vi.fn();
    render(
      <PeopleTable
        data={[mockPerson]}
        total={50}
        page={1}
        pageSize={25}
        onPageChange={onPageChange}
      />,
    );

    await user.click(screen.getByText("Previous"));
    expect(onPageChange).toHaveBeenCalledWith(0);
  });
});
