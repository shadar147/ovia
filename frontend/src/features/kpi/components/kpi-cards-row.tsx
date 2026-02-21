"use client";

import type { KpiSnapshot } from "@/lib/api/types";
import { KpiCard } from "./kpi-card";
import { HealthScoreBadge } from "./health-score-badge";
import { Heart, ShieldAlert, Zap, Clock } from "lucide-react";
import { useTranslation } from "@/i18n";

interface KpiCardsRowProps {
  latest: KpiSnapshot;
  previous?: KpiSnapshot;
}

export function KpiCardsRow({ latest, previous }: KpiCardsRowProps) {
  const { t } = useTranslation();

  const healthDelta = computeDelta(latest.delivery_health_score, previous?.delivery_health_score);
  const riskDelta = computeDelta(latest.release_risk_score, previous?.release_risk_score);
  const throughputDelta = previous ? latest.throughput_total - previous.throughput_total : null;
  const latencyDelta = computeDelta(latest.review_latency_median_hours, previous?.review_latency_median_hours);

  return (
    <div className="grid grid-cols-2 gap-4 lg:grid-cols-4">
      <div className="relative">
        <KpiCard
          title={t("kpi.deliveryHealth")}
          value={latest.delivery_health_score !== null ? latest.delivery_health_score.toFixed(1) : "N/A"}
          subtitle={latest.delivery_health_score !== null ? undefined : t("kpi.noData")}
          description={t("kpi.deliveryHealthDesc")}
          delta={healthDelta}
          deltaLabel="pts"
          icon={Heart}
          iconColor="text-green-600"
        />
        <div className="absolute right-4 top-4">
          <HealthScoreBadge score={latest.delivery_health_score} />
        </div>
      </div>
      <KpiCard
        title={t("kpi.releaseRisk")}
        value={latest.release_risk_score !== null ? latest.release_risk_score.toFixed(1) : "N/A"}
        subtitle={t("kpi.lowerIsBetter")}
        description={t("kpi.releaseRiskDesc")}
        delta={riskDelta !== null ? -riskDelta : null}
        deltaLabel="pts"
        icon={ShieldAlert}
        iconColor="text-orange-500"
      />
      <KpiCard
        title={t("kpi.throughput")}
        value={String(latest.throughput_total)}
        subtitle={t("kpi.itemsDelivered")}
        description={t("kpi.throughputDesc")}
        delta={throughputDelta}
        deltaLabel="items"
        icon={Zap}
        iconColor="text-blue-500"
      />
      <KpiCard
        title={t("kpi.reviewLatency")}
        value={latest.review_latency_median_hours !== null ? `${latest.review_latency_median_hours.toFixed(1)}h` : "N/A"}
        subtitle={latest.review_latency_p90_hours !== null ? `P90: ${latest.review_latency_p90_hours.toFixed(1)}h` : undefined}
        description={t("kpi.latencyDesc")}
        delta={latencyDelta !== null ? -latencyDelta : null}
        deltaLabel="hrs"
        icon={Clock}
        iconColor="text-purple-500"
      />
    </div>
  );
}

function computeDelta(current: number | null | undefined, previous: number | null | undefined): number | null {
  if (current === null || current === undefined || previous === null || previous === undefined) return null;
  return current - previous;
}
