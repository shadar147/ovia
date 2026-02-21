"use client";

import { Button } from "@/components/ui/button";
import { useTranslation } from "@/i18n";

interface ErrorStateProps {
  message?: string;
  onRetry?: () => void;
}

export function ErrorState({
  message,
  onRetry,
}: ErrorStateProps) {
  const { t } = useTranslation();

  return (
    <div className="flex flex-col items-center justify-center py-16 text-center">
      <h3 className="text-lg font-medium text-destructive">{t("state.error")}</h3>
      <p className="mt-1 text-sm text-muted-foreground">{message ?? t("state.somethingWrong")}</p>
      {onRetry && (
        <Button variant="outline" size="sm" className="mt-4" onClick={onRetry}>
          {t("state.retry")}
        </Button>
      )}
    </div>
  );
}
