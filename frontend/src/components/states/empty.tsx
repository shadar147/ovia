"use client";

import { useTranslation } from "@/i18n";

interface EmptyStateProps {
  title?: string;
  description?: string;
  action?: React.ReactNode;
}

export function EmptyState({
  title,
  description,
  action,
}: EmptyStateProps) {
  const { t } = useTranslation();

  return (
    <div className="flex flex-col items-center justify-center py-16 text-center">
      <h3 className="text-lg font-medium">{title ?? t("state.noData")}</h3>
      <p className="mt-1 text-sm text-muted-foreground">{description ?? t("state.nothingToShow")}</p>
      {action && <div className="mt-4">{action}</div>}
    </div>
  );
}
