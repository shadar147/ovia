import { describe, it, expect, vi, beforeEach } from "vitest";
import { render, screen } from "@testing-library/react";
import userEvent from "@testing-library/user-event";
import { LinkIdentityDialog } from "./link-identity-dialog";

const mockMutate = vi.fn();

vi.mock("@/features/people/hooks/use-people", () => ({
  useLinkIdentity: () => ({
    mutate: mockMutate,
    isPending: false,
  }),
}));

describe("LinkIdentityDialog", () => {
  beforeEach(() => {
    vi.clearAllMocks();
  });

  it("renders dialog with title and input", () => {
    render(
      <LinkIdentityDialog
        personId="p1"
        open={true}
        onOpenChange={() => {}}
      />,
    );

    expect(screen.getByText("Link Identity")).toBeInTheDocument();
    expect(screen.getByPlaceholderText(/Enter identity UUID/)).toBeInTheDocument();
  });

  it("disables submit when input is empty", () => {
    render(
      <LinkIdentityDialog
        personId="p1"
        open={true}
        onOpenChange={() => {}}
      />,
    );

    const submitBtn = screen.getByText("Link");
    expect(submitBtn).toBeDisabled();
  });

  it("disables submit for invalid UUID", async () => {
    const user = userEvent.setup();
    render(
      <LinkIdentityDialog
        personId="p1"
        open={true}
        onOpenChange={() => {}}
      />,
    );

    await user.type(screen.getByPlaceholderText(/Enter identity UUID/), "not-a-uuid");
    const submitBtn = screen.getByText("Link");
    expect(submitBtn).toBeDisabled();
  });

  it("enables submit for valid UUID and calls mutate", async () => {
    const user = userEvent.setup();
    render(
      <LinkIdentityDialog
        personId="p1"
        open={true}
        onOpenChange={() => {}}
      />,
    );

    const input = screen.getByPlaceholderText(/Enter identity UUID/);
    await user.type(input, "12345678-1234-1234-1234-123456789abc");

    const submitBtn = screen.getByText("Link");
    expect(submitBtn).not.toBeDisabled();

    await user.click(submitBtn);
    expect(mockMutate).toHaveBeenCalledWith(
      "12345678-1234-1234-1234-123456789abc",
      expect.any(Object),
    );
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
});
