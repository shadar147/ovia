"use client";

import { useState, useCallback } from "react";
import { Button } from "@/components/ui/button";
import { MappingTable } from "@/features/identity/components/mapping-table";
import { MappingFilters, defaultFilters, type FilterValues } from "@/features/identity/components/mapping-filters";
import { RuleTraceDrawer } from "@/features/identity/components/rule-trace-drawer";
import { LoadingState } from "@/components/states/loading";
import { ErrorState } from "@/components/states/error";
import { EmptyState } from "@/components/states/empty";
import {
  useIdentityMappings,
  useConfirmMapping,
  useSplitMapping,
  useBulkConfirm,
} from "@/features/identity/hooks/use-identity-mappings";
import type { PersonIdentityLink, IdentityMappingFilter } from "@/lib/api/types";
import { Download } from "lucide-react";
import { identityApi } from "@/features/identity/api/client";
import { toast } from "sonner";
import { useTranslation } from "@/i18n";

function buildApiFilter(f: FilterValues): IdentityMappingFilter {
  return {
    status: f.status === "all" ? undefined : f.status,
    min_confidence: f.minConfidence > 0 ? f.minConfidence / 100 : undefined,
    max_confidence: f.maxConfidence < 100 ? f.maxConfidence / 100 : undefined,
    limit: 200,
  };
}

export default function IdentityMappingPage() {
  const [filters, setFilters] = useState<FilterValues>(defaultFilters);
  const [selectedLink, setSelectedLink] = useState<PersonIdentityLink | null>(null);
  const [drawerOpen, setDrawerOpen] = useState(false);
  const { t } = useTranslation();

  const apiFilter = buildApiFilter(filters);
  const { data, isLoading, isError, error, refetch } = useIdentityMappings(apiFilter);

  const confirmMutation = useConfirmMapping();
  const splitMutation = useSplitMapping();
  const bulkConfirmMutation = useBulkConfirm();

  const handleRowClick = useCallback((link: PersonIdentityLink) => {
    setSelectedLink(link);
    setDrawerOpen(true);
  }, []);

  const handleConfirm = useCallback(
    (linkId: string) => {
      confirmMutation.mutate({ linkId, verifiedBy: "admin" });
      setDrawerOpen(false);
    },
    [confirmMutation],
  );

  const handleSplit = useCallback(
    (linkId: string) => {
      splitMutation.mutate({ linkId, verifiedBy: "admin" });
      setDrawerOpen(false);
    },
    [splitMutation],
  );

  const handleRemap = useCallback((_linkId: string) => {
    toast.info(t("identity.remapSoon"));
  }, [t]);

  const handleBulkConfirm = useCallback(() => {
    if (!data) return;
    const autoLinks = data.filter((l) => l.status === "auto").map((l) => l.id);
    if (autoLinks.length === 0) {
      toast.info(t("identity.noAutoLinks"));
      return;
    }
    bulkConfirmMutation.mutate({ linkIds: autoLinks, verifiedBy: "admin" });
  }, [data, bulkConfirmMutation, t]);

  const handleExportCsv = useCallback(async () => {
    try {
      const csv = await identityApi.exportCsv();
      const blob = new Blob([csv], { type: "text/csv" });
      const url = URL.createObjectURL(blob);
      const a = document.createElement("a");
      a.href = url;
      a.download = "conflicts.csv";
      a.click();
      URL.revokeObjectURL(url);
    } catch {
      toast.error(t("identity.exportFailed"));
    }
  }, [t]);

  const isPending = confirmMutation.isPending || splitMutation.isPending;
  const autoCount = data?.filter((l) => l.status === "auto").length ?? 0;

  return (
    <div className="space-y-6">
      <div className="flex items-center justify-between">
        <div>
          <h1 className="text-2xl font-semibold">{t("identity.title")}</h1>
          <p className="text-sm text-muted-foreground">
            {t("identity.subtitle")}
          </p>
        </div>
        <div className="flex gap-2">
          {autoCount > 0 && (
            <Button
              variant="outline"
              size="sm"
              onClick={handleBulkConfirm}
              disabled={bulkConfirmMutation.isPending}
            >
              {t("identity.bulkConfirm", { count: autoCount })}
            </Button>
          )}
          <Button variant="outline" size="sm" onClick={handleExportCsv}>
            <Download className="mr-1 h-4 w-4" />
            {t("identity.exportCsv")}
          </Button>
        </div>
      </div>

      <MappingFilters
        value={filters}
        onChange={setFilters}
        onReset={() => setFilters(defaultFilters)}
      />

      {isLoading ? (
        <LoadingState rows={8} />
      ) : isError ? (
        <ErrorState
          message={error instanceof Error ? error.message : t("identity.failedToLoad")}
          onRetry={() => refetch()}
        />
      ) : !data || data.length === 0 ? (
        <EmptyState
          title={t("identity.noMappings")}
          description={t("identity.noMappingsDesc")}
        />
      ) : (
        <MappingTable data={data} onRowClick={handleRowClick} />
      )}

      <RuleTraceDrawer
        link={selectedLink}
        open={drawerOpen}
        onClose={() => setDrawerOpen(false)}
        onConfirm={handleConfirm}
        onSplit={handleSplit}
        onRemap={handleRemap}
        isPending={isPending}
      />
    </div>
  );
}
