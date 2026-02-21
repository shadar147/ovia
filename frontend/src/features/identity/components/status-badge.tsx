import { Badge } from "@/components/ui/badge";
import type { LinkStatus } from "@/lib/api/types";

const statusConfig: Record<LinkStatus, { label: string; variant: "default" | "secondary" | "destructive" | "outline" }> = {
  auto: { label: "Auto", variant: "secondary" },
  verified: { label: "Verified", variant: "default" },
  conflict: { label: "Conflict", variant: "destructive" },
  rejected: { label: "Rejected", variant: "outline" },
  split: { label: "Split", variant: "outline" },
};

export function StatusBadge({ status }: { status: LinkStatus }) {
  const config = statusConfig[status] ?? { label: status, variant: "outline" as const };
  return <Badge variant={config.variant}>{config.label}</Badge>;
}

export function ConfidenceBadge({ confidence }: { confidence: number }) {
  const pct = Math.round(confidence * 100);
  const variant = pct >= 85 ? "default" : pct >= 50 ? "secondary" : "outline";
  return <Badge variant={variant}>{pct}%</Badge>;
}
