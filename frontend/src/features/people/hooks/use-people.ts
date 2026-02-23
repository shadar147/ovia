import { useQuery, useMutation, useQueryClient } from "@tanstack/react-query";
import { peopleApi } from "@/features/people/api/client";
import { peopleKeys } from "@/features/people/api/keys";
import type { PersonFilter, CreatePersonRequest } from "@/lib/api/types";
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
