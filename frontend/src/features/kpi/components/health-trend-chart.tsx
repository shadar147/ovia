"use client";

import ReactEChartsCore from "echarts-for-react/lib/core";
import * as echarts from "echarts/core";
import { LineChart } from "echarts/charts";
import { GridComponent, TooltipComponent, MarkLineComponent } from "echarts/components";
import { CanvasRenderer } from "echarts/renderers";
import type { KpiSnapshot } from "@/lib/api/types";
import { chartColors } from "./chart-colors";
import { formatWeekLabel } from "./chart-utils";
import { useTranslation } from "@/i18n";

echarts.use([LineChart, GridComponent, TooltipComponent, MarkLineComponent, CanvasRenderer]);

interface HealthTrendChartProps {
  history: KpiSnapshot[];
}

export function HealthTrendChart({ history }: HealthTrendChartProps) {
  const { t, locale } = useTranslation();

  if (history.length === 0) {
    return (
      <div className="flex h-[280px] items-center justify-center text-center">
        <div>
          <p className="text-sm font-medium text-muted-foreground">{t("dashboard.noHistory")}</p>
          <p className="mt-1 text-xs text-muted-foreground/70">{t("dashboard.noHistoryDesc")}</p>
        </div>
      </div>
    );
  }

  const sorted = [...history].sort(
    (a, b) => new Date(a.period_start).getTime() - new Date(b.period_start).getTime(),
  );

  const option: echarts.EChartsCoreOption = {
    tooltip: {
      trigger: "axis",
      valueFormatter: (val: number | null) => val != null ? `${val}` : "N/A",
    },
    grid: { left: 40, right: 16, top: 16, bottom: 24 },
    xAxis: {
      type: "category",
      data: sorted.map((s) => formatWeekLabel(s.period_start, locale)),
      axisLine: { lineStyle: { color: chartColors.gridLine } },
      axisLabel: { color: chartColors.text, fontSize: 11 },
    },
    yAxis: {
      type: "value",
      min: 0,
      max: 100,
      splitLine: { lineStyle: { color: chartColors.gridLine } },
      axisLabel: { color: chartColors.text, fontSize: 11 },
    },
    series: [
      {
        name: t("chart.deliveryHealth"),
        type: "line",
        data: sorted.map((s) => s.delivery_health_score),
        smooth: true,
        itemStyle: { color: chartColors.health },
        lineStyle: { width: 2.5 },
        areaStyle: { color: chartColors.healthArea },
        markLine: {
          silent: true,
          symbol: "none",
          lineStyle: { type: "dashed", width: 1 },
          data: [
            {
              yAxis: 80,
              label: { formatter: t("chart.healthyThreshold"), position: "insideEndTop", color: chartColors.thresholdGood, fontSize: 10 },
              lineStyle: { color: chartColors.thresholdGood },
            },
            {
              yAxis: 60,
              label: { formatter: t("chart.atRiskThreshold"), position: "insideEndTop", color: chartColors.thresholdWarn, fontSize: 10 },
              lineStyle: { color: chartColors.thresholdWarn },
            },
          ],
        },
      },
    ],
  };

  return <ReactEChartsCore echarts={echarts} option={option} style={{ height: 280 }} />;
}
