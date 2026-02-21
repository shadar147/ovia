import { useQuery } from "@tanstack/react-query";
import { kpiApi } from "@/features/kpi/api/client";
import { kpiKeys } from "@/features/kpi/api/keys";
import type { KpiHistoryFilter } from "@/lib/api/types";

export function useKpiLatest() {
  return useQuery({
    queryKey: kpiKeys.latest(),
    queryFn: () => kpiApi.getLatest(),
  });
}

export function useKpiHistory(filter: KpiHistoryFilter = {}) {
  return useQuery({
    queryKey: kpiKeys.history(filter),
    queryFn: () => kpiApi.listHistory(filter),
  });
}

export function useKpiRisks() {
  return useQuery({
    queryKey: kpiKeys.risks(),
    queryFn: () => kpiApi.listRisks(),
  });
}
