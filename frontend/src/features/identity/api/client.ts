import { api } from "@/lib/api/http";
import type {
  PersonIdentityLink,
  IdentityMappingFilter,
  ConflictQueueFilter,
  ConflictQueueStats,
  BulkConfirmResult,
} from "@/lib/api/types";

export const identityApi = {
  listMappings(filter: IdentityMappingFilter = {}) {
    const params = new URLSearchParams();
    if (filter.status) params.set("status", filter.status);
    if (filter.min_confidence !== undefined)
      params.set("min_confidence", String(filter.min_confidence));
    if (filter.max_confidence !== undefined)
      params.set("max_confidence", String(filter.max_confidence));
    if (filter.limit !== undefined) params.set("limit", String(filter.limit));
    if (filter.offset !== undefined) params.set("offset", String(filter.offset));

    const qs = params.toString();
    return api<PersonIdentityLink[]>(`/team/identity-mappings${qs ? `?${qs}` : ""}`);
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

  listConflicts(filter: ConflictQueueFilter = {}) {
    const params = new URLSearchParams();
    if (filter.sort) params.set("sort", filter.sort);
    if (filter.min_confidence !== undefined)
      params.set("min_confidence", String(filter.min_confidence));
    if (filter.max_confidence !== undefined)
      params.set("max_confidence", String(filter.max_confidence));
    if (filter.limit !== undefined) params.set("limit", String(filter.limit));
    if (filter.offset !== undefined) params.set("offset", String(filter.offset));

    const qs = params.toString();
    return api<PersonIdentityLink[]>(`/team/conflicts${qs ? `?${qs}` : ""}`);
  },

  bulkConfirm(linkIds: string[], verifiedBy: string) {
    return api<BulkConfirmResult>("/team/conflicts/bulk-confirm", {
      method: "POST",
      body: { link_ids: linkIds, verified_by: verifiedBy },
    });
  },

  conflictStats() {
    return api<ConflictQueueStats>("/team/conflicts/stats");
  },

  exportCsv() {
    return api<string>("/team/conflicts/export");
  },
};
