"use client";

import { Card, CardContent, CardDescription, CardHeader, CardTitle } from "@/components/ui/card";
import { ErrorState } from "@/components/states/error";
import { useKpiLatest, useKpiHistory, useKpiRisks } from "@/features/kpi/hooks/use-kpi";
import { KpiCardsRow } from "@/features/kpi/components/kpi-cards-row";
import { ThroughputChart } from "@/features/kpi/components/throughput-chart";
import { LatencyChart } from "@/features/kpi/components/latency-chart";
import { HealthTrendChart } from "@/features/kpi/components/health-trend-chart";
import { ThroughputBreakdown } from "@/features/kpi/components/throughput-breakdown";
import { RiskTable } from "@/features/kpi/components/risk-table";
import { DashboardSkeleton } from "@/features/kpi/components/dashboard-skeleton";
import { useTranslation, formatDateRange } from "@/i18n";

export default function DashboardPage() {
  const latest = useKpiLatest();
  const history = useKpiHistory({ limit: 12 });
  const risks = useKpiRisks();
  const { t, locale } = useTranslation();

  const isLoading = latest.isLoading || history.isLoading || risks.isLoading;
  const error = latest.error || history.error || risks.error;

  if (isLoading) {
    return (
      <div className="space-y-6">
        <div>
          <h1 className="text-2xl font-semibold">{t("dashboard.title")}</h1>
          <p className="mt-1 text-sm text-muted-foreground">{t("dashboard.loading")}</p>
        </div>
        <DashboardSkeleton />
      </div>
    );
  }

  if (error || !latest.data || !history.data) {
    return (
      <div className="space-y-6">
        <h1 className="text-2xl font-semibold">{t("dashboard.title")}</h1>
        <ErrorState
          message={error?.message ?? t("dashboard.failedToLoad")}
          onRetry={() => {
            latest.refetch();
            history.refetch();
            risks.refetch();
          }}
        />
      </div>
    );
  }

  const sortedHistory = [...history.data].sort(
    (a, b) => new Date(a.period_start).getTime() - new Date(b.period_start).getTime(),
  );
  const previous = sortedHistory.length >= 2 ? sortedHistory[sortedHistory.length - 2] : undefined;

  const periodLabel = formatDateRange(latest.data.period_start, latest.data.period_end, locale);

  return (
    <div className="space-y-6">
      {/* Header */}
      <div>
        <h1 className="text-2xl font-semibold">{t("dashboard.title")}</h1>
        <p className="mt-1 text-sm text-muted-foreground">
          {t("dashboard.weekOf", { period: periodLabel })}
        </p>
      </div>

      {/* KPI Cards */}
      <KpiCardsRow latest={latest.data} previous={previous} />

      {/* Charts Row: Throughput + Latency */}
      <div className="grid gap-4 lg:grid-cols-2">
        <Card>
          <CardHeader>
            <CardTitle className="text-base">{t("dashboard.throughputTrend")}</CardTitle>
            <CardDescription>
              {t("dashboard.throughputTrendDesc")}
            </CardDescription>
          </CardHeader>
          <CardContent>
            <ThroughputChart history={sortedHistory} />
          </CardContent>
        </Card>
        <Card>
          <CardHeader>
            <CardTitle className="text-base">{t("dashboard.latencyTrend")}</CardTitle>
            <CardDescription>
              {t("dashboard.latencyTrendDesc")}
            </CardDescription>
          </CardHeader>
          <CardContent>
            <LatencyChart history={sortedHistory} />
          </CardContent>
        </Card>
      </div>

      {/* Delivery Health Over Time */}
      <Card>
        <CardHeader>
          <CardTitle className="text-base">{t("dashboard.healthOverTime")}</CardTitle>
          <CardDescription>
            {t("dashboard.healthOverTimeDesc")}
          </CardDescription>
        </CardHeader>
        <CardContent>
          <HealthTrendChart history={sortedHistory} />
        </CardContent>
      </Card>

      {/* Bottom Row: Risks + Donut */}
      <div className="grid gap-4 lg:grid-cols-3">
        <Card className="lg:col-span-2">
          <CardHeader>
            <CardTitle className="text-base">{t("dashboard.topRisks")}</CardTitle>
            <CardDescription>
              {t("dashboard.topRisksDesc")}
            </CardDescription>
          </CardHeader>
          <CardContent>
            <RiskTable risks={risks.data ?? []} />
          </CardContent>
        </Card>
        <Card>
          <CardHeader>
            <CardTitle className="text-base">{t("dashboard.throughputMix")}</CardTitle>
            <CardDescription>
              {t("dashboard.throughputMixDesc")}
            </CardDescription>
          </CardHeader>
          <CardContent>
            <ThroughputBreakdown snapshot={latest.data} />
          </CardContent>
        </Card>
      </div>
    </div>
  );
}
