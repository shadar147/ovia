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
    // TODO: open remap modal with person selector
    toast.info("Remap modal coming soon");
  }, []);

  const handleBulkConfirm = useCallback(() => {
    if (!data) return;
    const autoLinks = data.filter((l) => l.status === "auto").map((l) => l.id);
    if (autoLinks.length === 0) {
      toast.info("No auto-matched links to confirm");
      return;
    }
    bulkConfirmMutation.mutate({ linkIds: autoLinks, verifiedBy: "admin" });
  }, [data, bulkConfirmMutation]);

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
      toast.error("Failed to export CSV");
    }
  }, []);

  const isPending = confirmMutation.isPending || splitMutation.isPending;
  const autoCount = data?.filter((l) => l.status === "auto").length ?? 0;

  return (
    <div className="space-y-6">
      <div className="flex items-center justify-between">
        <div>
          <h1 className="text-2xl font-semibold">Identity Mapping</h1>
          <p className="text-sm text-muted-foreground">
            Match identities across Jira, GitLab, and Confluence to canonical people.
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
              Bulk Confirm ({autoCount})
            </Button>
          )}
          <Button variant="outline" size="sm" onClick={handleExportCsv}>
            <Download className="mr-1 h-4 w-4" />
            Export CSV
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
          message={error instanceof Error ? error.message : "Failed to load mappings"}
          onRetry={() => refetch()}
        />
      ) : !data || data.length === 0 ? (
        <EmptyState
          title="No mappings"
          description="Run a sync first to populate identity mappings."
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
