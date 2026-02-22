"use client";

import ReactEChartsCore from "echarts-for-react/lib/core";
import * as echarts from "echarts/core";
import { LineChart } from "echarts/charts";
import { GridComponent, TooltipComponent, LegendComponent } from "echarts/components";
import { CanvasRenderer } from "echarts/renderers";
import type { KpiSnapshot } from "@/lib/api/types";
import { chartColors } from "./chart-colors";
import { formatWeekLabel } from "./chart-utils";
import { useTranslation } from "@/i18n";

echarts.use([LineChart, GridComponent, TooltipComponent, LegendComponent, CanvasRenderer]);

interface LatencyChartProps {
  history: KpiSnapshot[];
}

export function LatencyChart({ history }: LatencyChartProps) {
  const { t, locale } = useTranslation();

  if (history.length === 0) {
    return (
      <div className="flex h-[300px] items-center justify-center text-center">
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
      valueFormatter: (val: number | null) => val != null ? `${val}h` : "N/A",
    },
    legend: {
      bottom: 0,
      textStyle: { color: chartColors.text },
    },
    grid: { left: 40, right: 16, top: 16, bottom: 40 },
    xAxis: {
      type: "category",
      data: sorted.map((s) => formatWeekLabel(s.period_start, locale)),
      axisLine: { lineStyle: { color: chartColors.gridLine } },
      axisLabel: { color: chartColors.text, fontSize: 11 },
    },
    yAxis: {
      type: "value",
      name: t("chart.hours"),
      splitLine: { lineStyle: { color: chartColors.gridLine } },
      axisLabel: { color: chartColors.text, fontSize: 11 },
    },
    series: [
      {
        name: t("chart.median"),
        type: "line",
        data: sorted.map((s) => s.review_latency_median_hours),
        smooth: true,
        itemStyle: { color: chartColors.median },
        lineStyle: { width: 2 },
      },
      {
        name: t("chart.p90"),
        type: "line",
        data: sorted.map((s) => s.review_latency_p90_hours),
        smooth: true,
        itemStyle: { color: chartColors.p90 },
        lineStyle: { width: 2, type: "dashed" },
      },
    ],
  };

  return <ReactEChartsCore echarts={echarts} option={option} style={{ height: 300 }} />;
}
