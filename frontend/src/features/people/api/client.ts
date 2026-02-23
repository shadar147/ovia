import { api } from "@/lib/api/http";
import type {
  ListPeopleResponse,
  PersonResponse,
  PersonFilter,
  CreatePersonRequest,
  LinkedIdentitiesResponse,
  ActivityListResponse,
  ActivityFilter,
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

  get(id: string) {
    return api<PersonResponse>(`/team/people/${id}`);
  },

  listIdentities(personId: string) {
    return api<LinkedIdentitiesResponse>(
      `/team/people/${personId}/identities`,
    );
  },

  linkIdentity(personId: string, identityId: string) {
    return api<{ id: string; person_id: string; identity_id: string; status: string; confidence: number }>(
      `/team/people/${personId}/identities`,
      { method: "POST", body: { identity_id: identityId } },
    );
  },

  unlinkIdentity(personId: string, identityId: string) {
    return api<void>(
      `/team/people/${personId}/identities/${identityId}`,
      { method: "DELETE" },
    );
  },

  async listActivity(personId: string, filter: ActivityFilter = {}) {
    const params = new URLSearchParams();
    if (filter.period) params.set("period", filter.period);
    if (filter.source) params.set("source", filter.source);
    if (filter.type) params.set("type", filter.type);
    if (filter.limit !== undefined) params.set("limit", String(filter.limit));
    if (filter.offset !== undefined) params.set("offset", String(filter.offset));

    const qs = params.toString();
    return api<ActivityListResponse>(
      `/team/people/${personId}/activity${qs ? `?${qs}` : ""}`,
    );
  },
};
