"use client";

import ReactEChartsCore from "echarts-for-react/lib/core";
import * as echarts from "echarts/core";
import { BarChart } from "echarts/charts";
import { GridComponent, TooltipComponent, LegendComponent } from "echarts/components";
import { CanvasRenderer } from "echarts/renderers";
import type { KpiSnapshot } from "@/lib/api/types";
import { chartColors } from "./chart-colors";
import { formatWeekLabel } from "./chart-utils";
import { useTranslation } from "@/i18n";

echarts.use([BarChart, GridComponent, TooltipComponent, LegendComponent, CanvasRenderer]);

interface ThroughputChartProps {
  history: KpiSnapshot[];
}

export function ThroughputChart({ history }: ThroughputChartProps) {
  const { t, locale } = useTranslation();

  const sorted = [...history].sort(
    (a, b) => new Date(a.period_start).getTime() - new Date(b.period_start).getTime(),
  );

  const option: echarts.EChartsCoreOption = {
    tooltip: {
      trigger: "axis",
      axisPointer: { type: "shadow" },
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
      splitLine: { lineStyle: { color: chartColors.gridLine } },
      axisLabel: { color: chartColors.text, fontSize: 11 },
    },
    series: [
      {
        name: t("chart.features"),
        type: "bar",
        stack: "total",
        data: sorted.map((s) => s.throughput_features),
        itemStyle: { color: chartColors.features },
        barMaxWidth: 32,
      },
      {
        name: t("chart.bugs"),
        type: "bar",
        stack: "total",
        data: sorted.map((s) => s.throughput_bugs),
        itemStyle: { color: chartColors.bugs },
      },
      {
        name: t("chart.chores"),
        type: "bar",
        stack: "total",
        data: sorted.map((s) => s.throughput_chores),
        itemStyle: { color: chartColors.chores },
      },
    ],
  };

  return <ReactEChartsCore echarts={echarts} option={option} style={{ height: 300 }} />;
}
