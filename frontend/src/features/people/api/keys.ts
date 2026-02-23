import type { PersonFilter, ActivityFilter } from "@/lib/api/types";

export const peopleKeys = {
  all: ["people"] as const,
  list: (filter: PersonFilter = {}) => [...peopleKeys.all, "list", filter] as const,
  detail: (id: string) => [...peopleKeys.all, "detail", id] as const,
  identities: (id: string) => [...peopleKeys.all, "identities", id] as const,
  activity: (id: string, filter: ActivityFilter = {}) =>
    [...peopleKeys.all, "activity", id, filter] as const,
};
