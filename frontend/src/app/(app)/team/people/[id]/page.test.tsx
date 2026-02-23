import { describe, it, expect, vi, beforeEach } from "vitest";
import { render, screen } from "@testing-library/react";
import Person360Page from "./page";

// ── Mocks ────────────────────────────────────────────────────

const mockPerson = {
  id: "p1",
  display_name: "Alice Chen",
  primary_email: "alice@example.com",
  avatar_url: null,
  team: "Platform",
  role: "Engineer",
  status: "active",
  identity_count: 2,
  created_at: "2026-01-15T00:00:00Z",
  updated_at: "2026-02-20T00:00:00Z",
};

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
      source: "jira",
      username: null,
      email: "alice@jira.com",
      display_name: "Alice C",
      status: "auto",
      confidence: 0.85,
      linked_at: "2026-02-10T00:00:00Z",
    },
  ],
  count: 2,
};

const mockActivity = {
  data: [
    {
      id: "a1",
      source: "gitlab",
      type: "merge_request",
      title: "Fix login bug",
      url: "https://gitlab.com/mr/1",
      timestamp: "2026-02-18T14:30:00Z",
      metadata: { state: "merged", author: "alice.chen" },
    },
    {
      id: "a2",
      source: "identity",
      type: "identity_event",
      title: "Identity manual link",
      url: null,
      timestamp: "2026-02-01T00:00:00Z",
      metadata: {},
    },
  ],
  count: 2,
  total: 2,
};

const mockRefetch = vi.fn();
const mockUnlinkMutate = vi.fn();

let personReturn: Record<string, unknown> = {};
let identitiesReturn: Record<string, unknown> = {};
let activityReturn: Record<string, unknown> = {};

vi.mock("next/navigation", () => ({
  useParams: () => ({ id: "p1" }),
  useRouter: () => ({ push: vi.fn(), replace: vi.fn() }),
}));

vi.mock("@/features/people/hooks/use-people", () => ({
  usePerson: () => personReturn,
  usePersonIdentities: () => identitiesReturn,
  usePersonActivity: () => activityReturn,
  useUnlinkIdentity: () => ({
    mutate: mockUnlinkMutate,
    isPending: false,
  }),
  useLinkIdentity: () => ({
    mutate: vi.fn(),
    isPending: false,
  }),
  useOrphanIdentities: () => ({
    data: undefined,
    isLoading: false,
  }),
}));

vi.mock("next/link", () => ({
  default: ({
    children,
    href,
    ...props
  }: {
    children: React.ReactNode;
    href: string;
    [key: string]: unknown;
  }) => (
    <a href={href} {...props}>
      {children}
    </a>
  ),
}));

// ── Tests ────────────────────────────────────────────────────

describe("Person360Page", () => {
  beforeEach(() => {
    vi.clearAllMocks();
    personReturn = {
      data: mockPerson,
      isLoading: false,
      isError: false,
      error: null,
      refetch: mockRefetch,
    };
    identitiesReturn = {
      data: mockIdentities,
      isLoading: false,
    };
    activityReturn = {
      data: mockActivity,
      isLoading: false,
    };
  });

  it("renders person header with name, team, role, email", () => {
    render(<Person360Page />);

    // "Alice Chen" appears in header and identity card
    const aliceElements = screen.getAllByText("Alice Chen");
    expect(aliceElements.length).toBeGreaterThanOrEqual(1);
    expect(screen.getByText("Platform")).toBeInTheDocument();
    expect(screen.getByText("Engineer")).toBeInTheDocument();
    expect(screen.getByText("alice@example.com")).toBeInTheDocument();
    expect(screen.getByText("Active")).toBeInTheDocument();
  });

  it("renders linked identities with source group headers", () => {
    render(<Person360Page />);

    expect(screen.getByText("@alice.chen")).toBeInTheDocument();
    expect(screen.getByText("Alice C")).toBeInTheDocument();
    // Source group headers
    expect(screen.getByText("GitLab")).toBeInTheDocument();
    expect(screen.getByText("Jira")).toBeInTheDocument();
  });

  it("renders activity timeline items", () => {
    render(<Person360Page />);

    expect(screen.getByText("Fix login bug")).toBeInTheDocument();
    expect(screen.getByText("Identity manual link")).toBeInTheDocument();
  });

  it("shows MR link as clickable anchor", () => {
    render(<Person360Page />);

    const mrLink = screen.getByText("Fix login bug");
    expect(mrLink.closest("a")).toHaveAttribute(
      "href",
      "https://gitlab.com/mr/1",
    );
  });

  it("shows loading state when person is loading", () => {
    personReturn = {
      data: undefined,
      isLoading: true,
      isError: false,
      error: null,
      refetch: mockRefetch,
    };

    render(<Person360Page />);

    // Should show skeleton loading indicators
    const skeletons = document.querySelectorAll('[data-slot="skeleton"]');
    expect(skeletons.length).toBeGreaterThan(0);
  });

  it("shows error state with retry when person fails to load", () => {
    personReturn = {
      data: undefined,
      isLoading: false,
      isError: true,
      error: new Error("Network error"),
      refetch: mockRefetch,
    };

    render(<Person360Page />);

    expect(screen.getByText("Network error")).toBeInTheDocument();
    expect(screen.getByText("Retry")).toBeInTheDocument();
  });

  it("shows empty state for identities when none linked", () => {
    identitiesReturn = {
      data: { data: [], count: 0 },
      isLoading: false,
    };

    render(<Person360Page />);

    expect(screen.getByText("No identities linked yet")).toBeInTheDocument();
  });

  it("shows empty state for activity when none exists", () => {
    activityReturn = {
      data: { data: [], count: 0, total: 0 },
      isLoading: false,
    };

    render(<Person360Page />);

    expect(screen.getByText("No activity found")).toBeInTheDocument();
  });

  it("shows back-to-people link", () => {
    render(<Person360Page />);

    const backLink = screen.getByText("Back to People");
    expect(backLink.closest("a")).toHaveAttribute("href", "/team/people");
  });

  it("shows unlink confirmation dialog when X clicked", async () => {
    const { default: userEvent } = await import("@testing-library/user-event");
    const user = userEvent.setup();
    render(<Person360Page />);

    // Find the unlink buttons (X icons)
    const unlinkButtons = screen.getAllByTitle("Unlink");
    expect(unlinkButtons.length).toBe(2);

    await user.click(unlinkButtons[0] as HTMLElement);

    // Should show confirmation dialog instead of directly calling mutate
    expect(screen.getByText("Unlink this identity?")).toBeInTheDocument();
  });

  it("renders stats with identity count and total activity", () => {
    render(<Person360Page />);

    // Stats section
    expect(screen.getByText("Stats")).toBeInTheDocument();
    expect(screen.getByText("Total Activity")).toBeInTheDocument();
  });

  it("shows inactive badge when person is inactive", () => {
    personReturn = {
      ...personReturn,
      data: { ...mockPerson, status: "inactive" },
    };

    render(<Person360Page />);

    expect(screen.getByText("Inactive")).toBeInTheDocument();
  });

  it("renders identity link panel with panel header", () => {
    render(<Person360Page />);

    expect(screen.getByText("Linked Identities")).toBeInTheDocument();
    expect(screen.getByText("Link Identity")).toBeInTheDocument();
  });

  it("shows identity confidence for auto-matched links", () => {
    render(<Person360Page />);

    // Auto identity at 0.85 confidence shows 85%
    expect(screen.getByText("85%")).toBeInTheDocument();
  });
});
