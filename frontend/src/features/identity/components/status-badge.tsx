"use client";

import { Badge } from "@/components/ui/badge";
import type { LinkStatus } from "@/lib/api/types";
import { useTranslation } from "@/i18n";
import type { MessageKey } from "@/i18n";

const statusConfig: Record<LinkStatus, { labelKey: MessageKey; variant: "default" | "secondary" | "destructive" | "outline" }> = {
  auto: { labelKey: "status.auto", variant: "secondary" },
  verified: { labelKey: "status.verified", variant: "default" },
  conflict: { labelKey: "status.conflict", variant: "destructive" },
  rejected: { labelKey: "status.rejected", variant: "outline" },
  split: { labelKey: "status.split", variant: "outline" },
};

export function StatusBadge({ status }: { status: LinkStatus }) {
  const { t } = useTranslation();
  const config = statusConfig[status] ?? { labelKey: status as MessageKey, variant: "outline" as const };
  return <Badge variant={config.variant}>{t(config.labelKey)}</Badge>;
}

export function ConfidenceBadge({ confidence }: { confidence: number }) {
  const pct = Math.round(confidence * 100);
  const variant = pct >= 85 ? "default" : pct >= 50 ? "secondary" : "outline";
  return <Badge variant={variant}>{pct}%</Badge>;
}
