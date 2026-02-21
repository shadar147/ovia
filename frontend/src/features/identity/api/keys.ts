import type { IdentityMappingFilter, ConflictQueueFilter } from "@/lib/api/types";

export const identityKeys = {
  all: ["identity"] as const,
  mappings: (filter: IdentityMappingFilter = {}) => [...identityKeys.all, "mappings", filter] as const,
  conflicts: (filter: ConflictQueueFilter = {}) => [...identityKeys.all, "conflicts", filter] as const,
  conflictStats: () => [...identityKeys.all, "conflict-stats"] as const,
};
