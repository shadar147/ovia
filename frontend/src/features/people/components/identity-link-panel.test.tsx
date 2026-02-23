import { describe, it, expect, vi, beforeEach } from "vitest";
import { render, screen } from "@testing-library/react";
import userEvent from "@testing-library/user-event";
import { IdentityLinkPanel } from "./identity-link-panel";

const mockUnlinkMutate = vi.fn();
const mockLinkMutate = vi.fn();

const mockIdentities = {
  data: [
    {
      link_id: "l1",
      identity_id: "i1",
      source: "gitlab",
      username: "alice.chen",
      email: "alice@gitlab.com",
      display_name: "Alice Chen",
      status: "verified",
      confidence: 1.0,
      linked_at: "2026-02-01T00:00:00Z",
    },
    {
      link_id: "l2",
      identity_id: "i2",
      source: "gitlab",
      username: "alice.chen-2",
      email: "alice2@gitlab.com",
      display_name: "Alice Chen Alt",
      status: "auto",
      confidence: 0.85,
      linked_at: "2026-02-05T00:00:00Z",
    },
    {
      link_id: "l3",
      identity_id: "i3",
      source: "jira",
      username: null,
      email: "alice@jira.com",
      display_name: "Alice C",
      status: "verified",
      confidence: 1.0,
      linked_at: "2026-02-10T00:00:00Z",
    },
  ],
  count: 3,
};

let identitiesReturn: Record<string, unknown> = {};

vi.mock("@/features/people/hooks/use-people", () => ({
  usePersonIdentities: () => identitiesReturn,
  useUnlinkIdentity: () => ({
    mutate: mockUnlinkMutate,
    isPending: false,
  }),
  useLinkIdentity: () => ({
    mutate: mockLinkMutate,
    isPending: false,
  }),
  useOrphanIdentities: () => ({
    data: undefined,
    isLoading: false,
  }),
}));

describe("IdentityLinkPanel", () => {
  beforeEach(() => {
    vi.clearAllMocks();
    identitiesReturn = {
      data: mockIdentities,
      isLoading: false,
    };
  });

  it("renders identities grouped by source", () => {
    render(<IdentityLinkPanel personId="p1" identityCount={3} />);

    // Source group headers
    expect(screen.getByText("GitLab")).toBeInTheDocument();
    expect(screen.getByText("Jira")).toBeInTheDocument();

    // Group counts
    expect(screen.getByText("(2)")).toBeInTheDocument(); // 2 gitlab
    expect(screen.getByText("(1)")).toBeInTheDocument(); // 1 jira

    // Identity details
    expect(screen.getByText("Alice Chen")).toBeInTheDocument();
    expect(screen.getByText("Alice Chen Alt")).toBeInTheDocument();
    expect(screen.getByText("Alice C")).toBeInTheDocument();
    expect(screen.getByText("@alice.chen")).toBeInTheDocument();
  });

  it("shows status badges for each identity", () => {
    render(<IdentityLinkPanel personId="p1" identityCount={3} />);

    const verifiedBadges = screen.getAllByText("Verified");
    expect(verifiedBadges.length).toBe(2); // 2 verified identities
    expect(screen.getByText("Auto")).toBeInTheDocument(); // 1 auto identity
  });

  it("shows confidence percentage for non-100% identities", () => {
    render(<IdentityLinkPanel personId="p1" identityCount={3} />);

    expect(screen.getByText("85%")).toBeInTheDocument(); // 0.85 confidence
  });

  it("shows unlink confirmation dialog when X clicked", async () => {
    const user = userEvent.setup();
    render(<IdentityLinkPanel personId="p1" identityCount={3} />);

    const unlinkButtons = screen.getAllByTitle("Unlink");
    expect(unlinkButtons.length).toBe(3);

    await user.click(unlinkButtons[0] as HTMLElement);

    // Confirmation dialog appears
    expect(screen.getByText("Unlink this identity?")).toBeInTheDocument();
    expect(
      screen.getByText(/will be removed from the person/),
    ).toBeInTheDocument();
  });

  it("calls unlink mutate when confirmation confirmed", async () => {
    const user = userEvent.setup();
    render(<IdentityLinkPanel personId="p1" identityCount={3} />);

    // Click unlink on first identity
    const unlinkButtons = screen.getAllByTitle("Unlink");
    await user.click(unlinkButtons[0] as HTMLElement);

    // Click the destructive unlink button in the dialog
    const confirmBtns = screen.getAllByText("Unlink");
    // The dialog footer button (destructive variant)
    const dialogBtn = confirmBtns.find(
      (el) => el.closest("button")?.className.includes("destructive"),
    ) as HTMLElement;
    await user.click(dialogBtn);

    expect(mockUnlinkMutate).toHaveBeenCalledWith(
      "i1",
      expect.any(Object),
    );
  });

  it("cancels unlink when cancel clicked", async () => {
    const user = userEvent.setup();
    render(<IdentityLinkPanel personId="p1" identityCount={3} />);

    const unlinkButtons = screen.getAllByTitle("Unlink");
    await user.click(unlinkButtons[0] as HTMLElement);

    await user.click(screen.getByText("Cancel"));

    // Dialog should close, no mutate call
    expect(mockUnlinkMutate).not.toHaveBeenCalled();
  });

  it("shows empty state when no identities", () => {
    identitiesReturn = {
      data: { data: [], count: 0 },
      isLoading: false,
    };

    render(<IdentityLinkPanel personId="p1" identityCount={0} />);

    expect(screen.getByText("No identities linked yet")).toBeInTheDocument();
  });

  it("shows link identity button", () => {
    render(<IdentityLinkPanel personId="p1" identityCount={3} />);

    expect(screen.getByText("Link Identity")).toBeInTheDocument();
  });

  it("shows loading state while identities load", () => {
    identitiesReturn = {
      data: undefined,
      isLoading: true,
    };

    render(<IdentityLinkPanel personId="p1" identityCount={3} />);

    const skeletons = document.querySelectorAll('[data-slot="skeleton"]');
    expect(skeletons.length).toBeGreaterThan(0);
  });

  it("shows identity count in card description", () => {
    render(<IdentityLinkPanel personId="p1" identityCount={3} />);

    expect(screen.getByText(/3 identities/i)).toBeInTheDocument();
  });
});
