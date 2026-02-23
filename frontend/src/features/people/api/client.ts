import { api } from "@/lib/api/http";
import type {
  ListPeopleResponse,
  PersonResponse,
  PersonFilter,
  CreatePersonRequest,
} from "@/lib/api/types";

export const peopleApi = {
  async list(filter: PersonFilter = {}) {
    const params = new URLSearchParams();
    if (filter.search) params.set("search", filter.search);
    if (filter.team) params.set("team", filter.team);
    if (filter.status) params.set("status", filter.status);
    if (filter.limit !== undefined) params.set("limit", String(filter.limit));
    if (filter.offset !== undefined) params.set("offset", String(filter.offset));

    const qs = params.toString();
    return api<ListPeopleResponse>(`/team/people${qs ? `?${qs}` : ""}`);
  },

  create(body: CreatePersonRequest) {
    return api<PersonResponse>("/team/people", {
      method: "POST",
      body,
    });
  },
};
