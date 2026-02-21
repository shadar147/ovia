"use client";

import ReactEChartsCore from "echarts-for-react/lib/core";
import * as echarts from "echarts/core";
import { PieChart } from "echarts/charts";
import { TooltipComponent, LegendComponent } from "echarts/components";
import { CanvasRenderer } from "echarts/renderers";
import type { KpiSnapshot } from "@/lib/api/types";
import { chartColors } from "./chart-colors";
import { useTranslation } from "@/i18n";

echarts.use([PieChart, TooltipComponent, LegendComponent, CanvasRenderer]);

interface ThroughputBreakdownProps {
  snapshot: KpiSnapshot;
}

export function ThroughputBreakdown({ snapshot }: ThroughputBreakdownProps) {
  const { t } = useTranslation();

  const option: echarts.EChartsCoreOption = {
    tooltip: {
      trigger: "item",
      formatter: "{b}: {c} ({d}%)",
    },
    legend: {
      bottom: 0,
      textStyle: { color: chartColors.text },
    },
    series: [
      {
        type: "pie",
        radius: ["45%", "72%"],
        center: ["50%", "45%"],
        avoidLabelOverlap: false,
        label: {
          show: true,
          position: "center",
          formatter: () => `${snapshot.throughput_total}`,
          fontSize: 24,
          fontWeight: "bold",
          color: chartColors.text,
        },
        emphasis: {
          label: { show: true, fontSize: 14, fontWeight: "bold" },
        },
        data: [
          { value: snapshot.throughput_features, name: t("chart.features"), itemStyle: { color: chartColors.features } },
          { value: snapshot.throughput_bugs, name: t("chart.bugs"), itemStyle: { color: chartColors.bugs } },
          { value: snapshot.throughput_chores, name: t("chart.chores"), itemStyle: { color: chartColors.chores } },
        ],
      },
    ],
  };

  return <ReactEChartsCore echarts={echarts} option={option} style={{ height: 300 }} />;
}
