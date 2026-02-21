import { Card, CardContent } from "@/components/ui/card";
import {
  Tooltip,
  TooltipContent,
  TooltipProvider,
  TooltipTrigger,
} from "@/components/ui/tooltip";
import { cn } from "@/lib/utils";
import { TrendingUp, TrendingDown, Minus, Info, type LucideIcon } from "lucide-react";
import { useTranslation } from "@/i18n";

interface KpiCardProps {
  title: string;
  value: string;
  subtitle?: string;
  description?: string;
  delta?: number | null;
  deltaLabel?: string;
  icon: LucideIcon;
  iconColor?: string;
}

export function KpiCard({
  title,
  value,
  subtitle,
  description,
  delta,
  deltaLabel,
  icon: Icon,
  iconColor = "text-muted-foreground",
}: KpiCardProps) {
  return (
    <Card>
      <CardContent className="pt-0">
        <div className="flex items-start justify-between">
          <div className="space-y-1">
            <div className="flex items-center gap-1">
              <p className="text-sm text-muted-foreground">{title}</p>
              {description && (
                <TooltipProvider>
                  <Tooltip>
                    <TooltipTrigger asChild>
                      <Info className="h-3.5 w-3.5 cursor-help text-muted-foreground/60" />
                    </TooltipTrigger>
                    <TooltipContent side="top" className="max-w-xs text-left whitespace-pre-line">
                      {description}
                    </TooltipContent>
                  </Tooltip>
                </TooltipProvider>
              )}
            </div>
            <p className="text-2xl font-bold tracking-tight">{value}</p>
            {subtitle && (
              <p className="text-xs text-muted-foreground">{subtitle}</p>
            )}
          </div>
          <div className={cn("rounded-md bg-muted p-2", iconColor)}>
            <Icon className="h-5 w-5" />
          </div>
        </div>
        {delta !== undefined && delta !== null && (
          <DeltaIndicator delta={delta} label={deltaLabel} />
        )}
      </CardContent>
    </Card>
  );
}

function DeltaIndicator({ delta, label }: { delta: number; label?: string }) {
  const { t } = useTranslation();
  const isPositive = delta > 0;
  const isNeutral = delta === 0;
  const Icon = isNeutral ? Minus : isPositive ? TrendingUp : TrendingDown;
  const color = isNeutral
    ? "text-muted-foreground"
    : isPositive
      ? "text-green-600"
      : "text-red-600";

  return (
    <div className={cn("mt-2 flex items-center gap-1 text-xs", color)}>
      <Icon className="h-3 w-3" />
      <span>
        {isPositive ? "+" : ""}
        {delta.toFixed(1)}
        {label ? ` ${label}` : ""}
      </span>
      <span className="text-muted-foreground">{t("kpi.vsPrevWeek")}</span>
    </div>
  );
}
