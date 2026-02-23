import { describe, it, expect, vi, beforeEach } from "vitest";
import { render, screen } from "@testing-library/react";
import userEvent from "@testing-library/user-event";
import { LinkIdentityDialog } from "./link-identity-dialog";

const mockMutate = vi.fn();
const mockOrphans = {
  data: [
    {
      id: "oid-1",
      source: "gitlab",
      external_id: "ext-1",
      username: "alice.chen",
      email: "alice@gitlab.com",
      display_name: "Alice Chen",
      is_service_account: false,
      first_seen_at: "2026-01-01T00:00:00Z",
      last_seen_at: "2026-02-15T00:00:00Z",
    },
    {
      id: "oid-2",
      source: "jira",
      external_id: "ext-2",
      username: null,
      email: "bob@jira.com",
      display_name: "Bob Smith",
      is_service_account: false,
      first_seen_at: "2026-01-10T00:00:00Z",
      last_seen_at: "2026-02-10T00:00:00Z",
    },
  ],
  count: 2,
  total: 2,
};

let orphanReturn: Record<string, unknown> = {};

vi.mock("@/features/people/hooks/use-people", () => ({
  useLinkIdentity: () => ({
    mutate: mockMutate,
    isPending: false,
  }),
  useOrphanIdentities: () => orphanReturn,
}));

describe("LinkIdentityDialog", () => {
  beforeEach(() => {
    vi.clearAllMocks();
    orphanReturn = {
      data: undefined,
      isLoading: false,
    };
  });

  it("renders dialog with title and search input", () => {
    render(
      <LinkIdentityDialog
        personId="p1"
        open={true}
        onOpenChange={() => {}}
      />,
    );

    expect(screen.getByText("Link Identity")).toBeInTheDocument();
    expect(
      screen.getByPlaceholderText(/Search by username/),
    ).toBeInTheDocument();
  });

  it("shows minimum characters message when search is empty", () => {
    render(
      <LinkIdentityDialog
        personId="p1"
        open={true}
        onOpenChange={() => {}}
      />,
    );

    expect(
      screen.getByText(/Type at least 2 characters/),
    ).toBeInTheDocument();
  });

  it("disables link button when no identity is selected", () => {
    render(
      <LinkIdentityDialog
        personId="p1"
        open={true}
        onOpenChange={() => {}}
      />,
    );

    const linkBtn = screen.getByText("Link");
    expect(linkBtn).toBeDisabled();
  });

  it("renders orphan identity results", async () => {
    orphanReturn = {
      data: mockOrphans,
      isLoading: false,
    };

    const user = userEvent.setup();
    render(
      <LinkIdentityDialog
        personId="p1"
        open={true}
        onOpenChange={() => {}}
      />,
    );

    // Type in search to trigger results display
    const input = screen.getByPlaceholderText(/Search by username/);
    await user.type(input, "alice");

    expect(screen.getByText("Alice Chen")).toBeInTheDocument();
    expect(screen.getByText("Bob Smith")).toBeInTheDocument();
    expect(screen.getByText("@alice.chen")).toBeInTheDocument();
    expect(screen.getByText("bob@jira.com")).toBeInTheDocument();
  });

  it("selects an identity and enables link button", async () => {
    orphanReturn = {
      data: mockOrphans,
      isLoading: false,
    };

    const user = userEvent.setup();
    render(
      <LinkIdentityDialog
        personId="p1"
        open={true}
        onOpenChange={() => {}}
      />,
    );

    await user.type(screen.getByPlaceholderText(/Search by username/), "alice");

    // Click on Alice to select
    await user.click(screen.getByText("Alice Chen"));

    const linkBtn = screen.getByText("Link");
    expect(linkBtn).not.toBeDisabled();
  });

  it("calls mutate with selected identity ID on link", async () => {
    orphanReturn = {
      data: mockOrphans,
      isLoading: false,
    };

    const user = userEvent.setup();
    render(
      <LinkIdentityDialog
        personId="p1"
        open={true}
        onOpenChange={() => {}}
      />,
    );

    await user.type(screen.getByPlaceholderText(/Search by username/), "alice");
    await user.click(screen.getByText("Alice Chen"));
    await user.click(screen.getByText("Link"));

    expect(mockMutate).toHaveBeenCalledWith("oid-1", expect.any(Object));
  });

  it("shows no results message when search returns empty", async () => {
    orphanReturn = {
      data: { data: [], count: 0, total: 0 },
      isLoading: false,
    };

    const user = userEvent.setup();
    render(
      <LinkIdentityDialog
        personId="p1"
        open={true}
        onOpenChange={() => {}}
      />,
    );

    await user.type(screen.getByPlaceholderText(/Search by username/), "nonexistent");

    expect(
      screen.getByText(/No unlinked identities found/),
    ).toBeInTheDocument();
  });

  it("shows loading skeletons while searching", async () => {
    orphanReturn = {
      data: undefined,
      isLoading: true,
    };

    const user = userEvent.setup();
    render(
      <LinkIdentityDialog
        personId="p1"
        open={true}
        onOpenChange={() => {}}
      />,
    );

    await user.type(screen.getByPlaceholderText(/Search by username/), "alice");

    const skeletons = document.querySelectorAll('[data-slot="skeleton"]');
    expect(skeletons.length).toBe(3);
  });

  it("does not render when closed", () => {
    render(
      <LinkIdentityDialog
        personId="p1"
        open={false}
        onOpenChange={() => {}}
      />,
    );

    expect(screen.queryByText("Link Identity")).not.toBeInTheDocument();
  });

  it("shows source filter dropdown", () => {
    render(
      <LinkIdentityDialog
        personId="p1"
        open={true}
        onOpenChange={() => {}}
      />,
    );

    expect(screen.getByText("All sources")).toBeInTheDocument();
  });

  it("shows service account badge for bot identities", async () => {
    orphanReturn = {
      data: {
        data: [
          {
            id: "oid-bot",
            source: "gitlab",
            external_id: "ext-bot",
            username: "ci-bot",
            email: null,
            display_name: "CI Bot",
            is_service_account: true,
            first_seen_at: null,
            last_seen_at: null,
          },
        ],
        count: 1,
        total: 1,
      },
      isLoading: false,
    };

    const user = userEvent.setup();
    render(
      <LinkIdentityDialog
        personId="p1"
        open={true}
        onOpenChange={() => {}}
      />,
    );

    await user.type(screen.getByPlaceholderText(/Search by username/), "ci-bot");

    expect(screen.getByText("bot")).toBeInTheDocument();
  });
});
