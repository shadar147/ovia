import { api } from "@/lib/api/http";
import type {
  PersonIdentityLink,
  IdentityMappingFilter,
  ConflictQueueFilter,
  ConflictQueueStats,
  BulkConfirmResult,
} from "@/lib/api/types";

export const identityApi = {
  async listMappings(filter: IdentityMappingFilter = {}) {
    const params = new URLSearchParams();
    if (filter.status) params.set("status", filter.status);
    if (filter.min_confidence !== undefined)
      params.set("min_confidence", String(filter.min_confidence));
    if (filter.max_confidence !== undefined)
      params.set("max_confidence", String(filter.max_confidence));
    if (filter.limit !== undefined) params.set("limit", String(filter.limit));
    if (filter.offset !== undefined) params.set("offset", String(filter.offset));

    const qs = params.toString();
    const res = await api<{ data: PersonIdentityLink[]; count: number }>(
      `/team/identity-mappings${qs ? `?${qs}` : ""}`,
    );
    return res.data;
  },

  confirmMapping(linkId: string, verifiedBy: string) {
    return api<void>("/team/identity-mappings/confirm", {
      method: "POST",
      body: { link_id: linkId, verified_by: verifiedBy },
    });
  },

  remapMapping(linkId: string, newPersonId: string, verifiedBy: string) {
    return api<void>("/team/identity-mappings/remap", {
      method: "POST",
      body: { link_id: linkId, new_person_id: newPersonId, verified_by: verifiedBy },
    });
  },

  splitMapping(linkId: string, verifiedBy: string) {
    return api<void>("/team/identity-mappings/split", {
      method: "POST",
      body: { link_id: linkId, verified_by: verifiedBy },
    });
  },

  async listConflicts(filter: ConflictQueueFilter = {}) {
    const params = new URLSearchParams();
    if (filter.sort) params.set("sort_by", filter.sort);
    if (filter.min_confidence !== undefined)
      params.set("min_confidence", String(filter.min_confidence));
    if (filter.max_confidence !== undefined)
      params.set("max_confidence", String(filter.max_confidence));
    if (filter.limit !== undefined) params.set("limit", String(filter.limit));
    if (filter.offset !== undefined) params.set("offset", String(filter.offset));

    const qs = params.toString();
    const res = await api<{ data: PersonIdentityLink[]; count: number }>(
      `/team/conflict-queue${qs ? `?${qs}` : ""}`,
    );
    return res.data;
  },

  bulkConfirm(linkIds: string[], verifiedBy: string) {
    return api<BulkConfirmResult>("/team/conflict-queue/bulk-confirm", {
      method: "POST",
      body: { link_ids: linkIds, verified_by: verifiedBy },
    });
  },

  conflictStats() {
    return api<ConflictQueueStats>("/team/conflict-queue/stats");
  },

  exportCsv() {
    return api<string>("/team/conflict-queue/export");
  },
};
