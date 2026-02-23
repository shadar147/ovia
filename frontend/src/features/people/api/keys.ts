import type { PersonFilter } from "@/lib/api/types";

export const peopleKeys = {
  all: ["people"] as const,
  list: (filter: PersonFilter = {}) => [...peopleKeys.all, "list", filter] as const,
  detail: (id: string) => [...peopleKeys.all, "detail", id] as const,
};
