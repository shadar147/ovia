"use client";

import { useState, useMemo, useCallback } from "react";
import { Button } from "@/components/ui/button";
import { Input } from "@/components/ui/input";
import {
  Select,
  SelectContent,
  SelectItem,
  SelectTrigger,
  SelectValue,
} from "@/components/ui/select";
import { LoadingState } from "@/components/states/loading";
import { ErrorState } from "@/components/states/error";
import { EmptyState } from "@/components/states/empty";
import { PeopleTable } from "@/features/people/components/people-table";
import { CreatePersonDialog } from "@/features/people/components/create-person-dialog";
import { usePeople } from "@/features/people/hooks/use-people";
import type { PersonFilter } from "@/lib/api/types";
import { useTranslation } from "@/i18n";
import { Plus, Search } from "lucide-react";

const PAGE_SIZE = 25;

export default function PeoplePage() {
  const { t } = useTranslation();
  const [search, setSearch] = useState("");
  const [statusFilter, setStatusFilter] = useState<string>("all");
  const [page, setPage] = useState(0);
  const [createOpen, setCreateOpen] = useState(false);

  const filter: PersonFilter = useMemo(
    () => ({
      search: search.trim() || undefined,
      status: statusFilter === "all" ? undefined : statusFilter,
      limit: PAGE_SIZE,
      offset: page * PAGE_SIZE,
    }),
    [search, statusFilter, page],
  );

  const { data, isLoading, isError, error, refetch } = usePeople(filter);

  const handleSearchChange = useCallback(
    (e: React.ChangeEvent<HTMLInputElement>) => {
      setSearch(e.target.value);
      setPage(0);
    },
    [],
  );

  const handleStatusChange = useCallback((value: string) => {
    setStatusFilter(value);
    setPage(0);
  }, []);

  const isEmpty = !isLoading && !isError && data && data.data.length === 0;
  const hasNoFilters = !search.trim() && statusFilter === "all";

  return (
    <div className="space-y-6">
      {/* Header */}
      <div className="flex items-center justify-between">
        <div>
          <h1 className="text-2xl font-semibold">{t("people.title")}</h1>
          <p className="text-sm text-muted-foreground">
            {t("people.subtitle")}
          </p>
        </div>
        <Button size="sm" onClick={() => setCreateOpen(true)}>
          <Plus className="mr-1 h-4 w-4" />
          {t("people.addPerson")}
        </Button>
      </div>

      {/* Filters */}
      <div className="flex items-center gap-3">
        <div className="relative flex-1 max-w-sm">
          <Search className="absolute left-2.5 top-2.5 h-4 w-4 text-muted-foreground" />
          <Input
            value={search}
            onChange={handleSearchChange}
            placeholder={t("people.search")}
            className="pl-9"
          />
        </div>
        <Select value={statusFilter} onValueChange={handleStatusChange}>
          <SelectTrigger className="w-[140px]">
            <SelectValue />
          </SelectTrigger>
          <SelectContent>
            <SelectItem value="all">{t("people.filterAll")}</SelectItem>
            <SelectItem value="active">{t("people.filterActive")}</SelectItem>
            <SelectItem value="inactive">
              {t("people.filterInactive")}
            </SelectItem>
          </SelectContent>
        </Select>
      </div>

      {/* Content */}
      {isLoading ? (
        <LoadingState rows={8} />
      ) : isError ? (
        <ErrorState
          message={
            error instanceof Error
              ? error.message
              : t("people.failedToLoad")
          }
          onRetry={() => refetch()}
        />
      ) : isEmpty && hasNoFilters ? (
        <EmptyState
          title={t("people.noPeople")}
          description={t("people.noPeopleDesc")}
          action={
            <Button size="sm" onClick={() => setCreateOpen(true)}>
              <Plus className="mr-1 h-4 w-4" />
              {t("people.addPerson")}
            </Button>
          }
        />
      ) : (
        <PeopleTable
          data={data?.data ?? []}
          total={data?.total ?? 0}
          page={page}
          pageSize={PAGE_SIZE}
          onPageChange={setPage}
        />
      )}

      {/* Create modal */}
      <CreatePersonDialog open={createOpen} onOpenChange={setCreateOpen} />
    </div>
  );
}
