import { api } from "@/lib/api/http";
import type { KpiSnapshot, RiskItem, KpiHistoryFilter } from "@/lib/api/types";

export const kpiApi = {
  async getLatest() {
    const res = await api<{ data: KpiSnapshot }>("/team/kpi");
    return res.data;
  },

  async listHistory(filter: KpiHistoryFilter = {}) {
    const params = new URLSearchParams();
    if (filter.period_start) params.set("period_start", filter.period_start);
    if (filter.period_end) params.set("period_end", filter.period_end);
    if (filter.limit !== undefined) params.set("limit", String(filter.limit));
    if (filter.offset !== undefined) params.set("offset", String(filter.offset));

    const qs = params.toString();
    const res = await api<{ data: KpiSnapshot[]; count: number }>(
      `/team/kpi/history${qs ? `?${qs}` : ""}`,
    );
    return res.data;
  },

  async listRisks() {
    const res = await api<{ data: RiskItem[]; count: number }>("/team/kpi/risks");
    return res.data;
  },
};
