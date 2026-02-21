"use client";

import { Badge } from "@/components/ui/badge";
import { cn } from "@/lib/utils";
import { useTranslation } from "@/i18n";

interface HealthScoreBadgeProps {
  score: number | null;
  className?: string;
}

export function HealthScoreBadge({ score, className }: HealthScoreBadgeProps) {
  const { t } = useTranslation();

  if (score === null) {
    return (
      <Badge variant="outline" className={cn("text-muted-foreground", className)}>
        N/A
      </Badge>
    );
  }

  const { label, colorClass } = getHealthLevel(score, t);

  return (
    <Badge className={cn(colorClass, className)}>
      {label}
    </Badge>
  );
}

function getHealthLevel(score: number, t: (key: "health.healthy" | "health.atRisk" | "health.critical") => string) {
  if (score >= 80) {
    return { label: t("health.healthy"), colorClass: "bg-green-600 text-white hover:bg-green-600" };
  }
  if (score >= 60) {
    return { label: t("health.atRisk"), colorClass: "bg-yellow-500 text-white hover:bg-yellow-500" };
  }
  return { label: t("health.critical"), colorClass: "bg-red-600 text-white hover:bg-red-600" };
}
