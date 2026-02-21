import { useQuery, useMutation, useQueryClient } from "@tanstack/react-query";
import { identityApi } from "@/features/identity/api/client";
import { identityKeys } from "@/features/identity/api/keys";
import type { IdentityMappingFilter, ConflictQueueFilter } from "@/lib/api/types";
import { toast } from "sonner";

export function useIdentityMappings(filter: IdentityMappingFilter = {}) {
  return useQuery({
    queryKey: identityKeys.mappings(filter),
    queryFn: () => identityApi.listMappings(filter),
  });
}

export function useConflicts(filter: ConflictQueueFilter = {}) {
  return useQuery({
    queryKey: identityKeys.conflicts(filter),
    queryFn: () => identityApi.listConflicts(filter),
  });
}

export function useConflictStats() {
  return useQuery({
    queryKey: identityKeys.conflictStats(),
    queryFn: () => identityApi.conflictStats(),
  });
}

export function useConfirmMapping() {
  const qc = useQueryClient();
  return useMutation({
    mutationFn: ({ linkId, verifiedBy }: { linkId: string; verifiedBy: string }) =>
      identityApi.confirmMapping(linkId, verifiedBy),
    onSuccess: () => {
      qc.invalidateQueries({ queryKey: identityKeys.all });
      toast.success("Mapping confirmed");
    },
    onError: () => toast.error("Failed to confirm mapping"),
  });
}

export function useRemapMapping() {
  const qc = useQueryClient();
  return useMutation({
    mutationFn: ({
      linkId,
      newPersonId,
      verifiedBy,
    }: {
      linkId: string;
      newPersonId: string;
      verifiedBy: string;
    }) => identityApi.remapMapping(linkId, newPersonId, verifiedBy),
    onSuccess: () => {
      qc.invalidateQueries({ queryKey: identityKeys.all });
      toast.success("Mapping remapped");
    },
    onError: () => toast.error("Failed to remap mapping"),
  });
}

export function useSplitMapping() {
  const qc = useQueryClient();
  return useMutation({
    mutationFn: ({ linkId, verifiedBy }: { linkId: string; verifiedBy: string }) =>
      identityApi.splitMapping(linkId, verifiedBy),
    onSuccess: () => {
      qc.invalidateQueries({ queryKey: identityKeys.all });
      toast.success("Mapping split");
    },
    onError: () => toast.error("Failed to split mapping"),
  });
}

export function useBulkConfirm() {
  const qc = useQueryClient();
  return useMutation({
    mutationFn: ({ linkIds, verifiedBy }: { linkIds: string[]; verifiedBy: string }) =>
      identityApi.bulkConfirm(linkIds, verifiedBy),
    onSuccess: (data) => {
      qc.invalidateQueries({ queryKey: identityKeys.all });
      toast.success(`Confirmed ${data.confirmed} mappings`);
    },
    onError: () => toast.error("Bulk confirm failed"),
  });
}
