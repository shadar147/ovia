import type { KpiHistoryFilter } from "@/lib/api/types";

export const kpiKeys = {
  all: ["kpi"] as const,
  latest: () => [...kpiKeys.all, "latest"] as const,
  history: (filter: KpiHistoryFilter = {}) => [...kpiKeys.all, "history", filter] as const,
  risks: () => [...kpiKeys.all, "risks"] as const,
};
