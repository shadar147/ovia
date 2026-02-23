import { useQuery, useMutation, useQueryClient } from "@tanstack/react-query";
import { peopleApi } from "@/features/people/api/client";
import { peopleKeys } from "@/features/people/api/keys";
import type { PersonFilter, CreatePersonRequest, ActivityFilter, OrphanIdentityFilter } from "@/lib/api/types";
import { toast } from "sonner";
import { useTranslation } from "@/i18n";

export function usePeople(filter: PersonFilter = {}) {
  return useQuery({
    queryKey: peopleKeys.list(filter),
    queryFn: () => peopleApi.list(filter),
  });
}

export function useCreatePerson() {
  const qc = useQueryClient();
  const { t } = useTranslation();

  return useMutation({
    mutationFn: (body: CreatePersonRequest) => peopleApi.create(body),
    onSuccess: () => {
      qc.invalidateQueries({ queryKey: peopleKeys.all });
      toast.success(t("people.created"));
    },
    onError: () => toast.error(t("people.createFailed")),
  });
}

export function usePerson(id: string) {
  return useQuery({
    queryKey: peopleKeys.detail(id),
    queryFn: () => peopleApi.get(id),
    enabled: !!id,
  });
}

export function usePersonIdentities(personId: string) {
  return useQuery({
    queryKey: peopleKeys.identities(personId),
    queryFn: () => peopleApi.listIdentities(personId),
    enabled: !!personId,
  });
}

export function useLinkIdentity(personId: string) {
  const qc = useQueryClient();
  const { t } = useTranslation();

  return useMutation({
    mutationFn: (identityId: string) =>
      peopleApi.linkIdentity(personId, identityId),
    onSuccess: () => {
      qc.invalidateQueries({ queryKey: peopleKeys.identities(personId) });
      qc.invalidateQueries({ queryKey: peopleKeys.detail(personId) });
      toast.success(t("person360.identityLinked"));
    },
    onError: () => toast.error(t("person360.linkFailed")),
  });
}

export function useUnlinkIdentity(personId: string) {
  const qc = useQueryClient();
  const { t } = useTranslation();

  return useMutation({
    mutationFn: (identityId: string) =>
      peopleApi.unlinkIdentity(personId, identityId),
    onSuccess: () => {
      qc.invalidateQueries({ queryKey: peopleKeys.identities(personId) });
      qc.invalidateQueries({ queryKey: peopleKeys.detail(personId) });
      toast.success(t("person360.identityUnlinked"));
    },
    onError: () => toast.error(t("person360.unlinkFailed")),
  });
}

export function useOrphanIdentities(filter: OrphanIdentityFilter = {}) {
  return useQuery({
    queryKey: peopleKeys.orphanIdentities(filter),
    queryFn: () => peopleApi.searchOrphanIdentities(filter),
    enabled: !!filter.search && filter.search.length >= 2,
  });
}

export function usePersonActivity(personId: string, filter: ActivityFilter = {}) {
  return useQuery({
    queryKey: peopleKeys.activity(personId, filter),
    queryFn: () => peopleApi.listActivity(personId, filter),
    enabled: !!personId,
  });
}
