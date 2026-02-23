"use client";

import { useState, useMemo, useCallback } from "react";
import {
  Dialog,
  DialogContent,
  DialogHeader,
  DialogTitle,
  DialogDescription,
  DialogFooter,
} from "@/components/ui/dialog";
import {
  Select,
  SelectContent,
  SelectItem,
  SelectTrigger,
  SelectValue,
} from "@/components/ui/select";
import { Button } from "@/components/ui/button";
import { Badge } from "@/components/ui/badge";
import { Input } from "@/components/ui/input";
import {
  useLinkIdentity,
  useOrphanIdentities,
} from "@/features/people/hooks/use-people";
import { useTranslation } from "@/i18n";
import { Search, GitMerge, Briefcase, FileText, GitCommit, Shield } from "lucide-react";
import type { OrphanIdentityResponse, OrphanIdentityFilter } from "@/lib/api/types";

interface LinkIdentityDialogProps {
  personId: string;
  open: boolean;
  onOpenChange: (open: boolean) => void;
}

const SOURCE_ICONS: Record<string, React.ElementType> = {
  gitlab: GitMerge,
  jira: Briefcase,
  confluence: FileText,
  git_commit_author: GitCommit,
  git_commit_committer: GitCommit,
  git_config_snapshot: GitCommit,
};

const SOURCE_COLORS: Record<string, string> = {
  gitlab: "bg-orange-100 text-orange-800 dark:bg-orange-900 dark:text-orange-200",
  jira: "bg-blue-100 text-blue-800 dark:bg-blue-900 dark:text-blue-200",
  confluence: "bg-teal-100 text-teal-800 dark:bg-teal-900 dark:text-teal-200",
};

export function LinkIdentityDialog({
  personId,
  open,
  onOpenChange,
}: LinkIdentityDialogProps) {
  const { t } = useTranslation();
  const linkMutation = useLinkIdentity(personId);
  const [searchText, setSearchText] = useState("");
  const [sourceFilter, setSourceFilter] = useState("all");
  const [selectedId, setSelectedId] = useState<string | null>(null);

  const filter: OrphanIdentityFilter = useMemo(
    () => ({
      search: searchText.trim(),
      source: sourceFilter === "all" ? undefined : sourceFilter,
      limit: 20,
    }),
    [searchText, sourceFilter],
  );

  const { data: orphans, isLoading } = useOrphanIdentities(filter);

  const handleLink = useCallback(() => {
    if (!selectedId) return;
    linkMutation.mutate(selectedId, {
      onSuccess: () => {
        setSearchText("");
        setSelectedId(null);
        setSourceFilter("all");
        onOpenChange(false);
      },
    });
  }, [selectedId, linkMutation, onOpenChange]);

  const handleClose = useCallback(
    (open: boolean) => {
      if (!open) {
        setSearchText("");
        setSelectedId(null);
        setSourceFilter("all");
      }
      onOpenChange(open);
    },
    [onOpenChange],
  );

  return (
    <Dialog open={open} onOpenChange={handleClose}>
      <DialogContent className="sm:max-w-lg">
        <DialogHeader>
          <DialogTitle>{t("person360.linkDialogTitle")}</DialogTitle>
          <DialogDescription>{t("person360.linkDialogSearchDesc")}</DialogDescription>
        </DialogHeader>

        <div className="space-y-3">
          {/* Search + Source filter row */}
          <div className="flex gap-2">
            <div className="relative flex-1">
              <Search className="absolute left-2.5 top-2.5 h-4 w-4 text-muted-foreground" />
              <Input
                value={searchText}
                onChange={(e) => {
                  setSearchText(e.target.value);
                  setSelectedId(null);
                }}
                placeholder={t("person360.searchIdentityPlaceholder")}
                className="pl-9"
                autoFocus
              />
            </div>
            <Select value={sourceFilter} onValueChange={setSourceFilter}>
              <SelectTrigger className="w-[120px]">
                <SelectValue />
              </SelectTrigger>
              <SelectContent>
                <SelectItem value="all">{t("person360.sourceAll")}</SelectItem>
                <SelectItem value="gitlab">{t("person360.sourceGitlab")}</SelectItem>
                <SelectItem value="jira">{t("person360.sourceJira")}</SelectItem>
                <SelectItem value="confluence">Confluence</SelectItem>
              </SelectContent>
            </Select>
          </div>

          {/* Results list */}
          <div className="max-h-64 overflow-y-auto rounded-md border">
            {searchText.trim().length < 2 ? (
              <div className="p-4 text-center text-sm text-muted-foreground">
                {t("person360.searchMinChars")}
              </div>
            ) : isLoading ? (
              <div className="space-y-2 p-3">
                {[1, 2, 3].map((i) => (
                  <div
                    key={i}
                    data-slot="skeleton"
                    className="h-12 animate-pulse rounded bg-muted"
                  />
                ))}
              </div>
            ) : !orphans?.data.length ? (
              <div className="p-4 text-center text-sm text-muted-foreground">
                {t("person360.noOrphansFound")}
              </div>
            ) : (
              <div className="divide-y">
                {orphans.data.map((identity) => (
                  <OrphanIdentityRow
                    key={identity.id}
                    identity={identity}
                    selected={selectedId === identity.id}
                    onSelect={() =>
                      setSelectedId((prev) =>
                        prev === identity.id ? null : identity.id,
                      )
                    }
                  />
                ))}
              </div>
            )}
          </div>

          {orphans && orphans.total > orphans.count && searchText.trim().length >= 2 && (
            <p className="text-xs text-muted-foreground text-right">
              {t("person360.showingOf")
                .replace("{count}", String(orphans.count))
                .replace("{total}", String(orphans.total))}
            </p>
          )}
        </div>

        <DialogFooter>
          <Button
            type="button"
            variant="outline"
            onClick={() => handleClose(false)}
          >
            {t("people.cancel")}
          </Button>
          <Button
            onClick={handleLink}
            disabled={!selectedId || linkMutation.isPending}
          >
            {linkMutation.isPending
              ? t("person360.linking")
              : t("person360.link")}
          </Button>
        </DialogFooter>
      </DialogContent>
    </Dialog>
  );
}

function OrphanIdentityRow({
  identity,
  selected,
  onSelect,
}: {
  identity: OrphanIdentityResponse;
  selected: boolean;
  onSelect: () => void;
}) {
  const Icon = SOURCE_ICONS[identity.source] ?? Shield;
  const color =
    SOURCE_COLORS[identity.source] ?? "bg-muted text-muted-foreground";

  return (
    <button
      type="button"
      onClick={onSelect}
      className={`flex w-full items-center gap-3 px-3 py-2.5 text-left transition-colors hover:bg-muted/50 ${
        selected ? "bg-primary/10 ring-1 ring-primary/30" : ""
      }`}
    >
      <span
        className={`inline-flex shrink-0 items-center gap-1 rounded px-1.5 py-0.5 text-xs font-medium ${color}`}
      >
        <Icon className="h-3 w-3" />
        {identity.source}
      </span>
      <div className="min-w-0 flex-1">
        <div className="flex items-center gap-2">
          <span className="truncate text-sm font-medium">
            {identity.display_name ?? identity.username ?? identity.email ?? identity.id}
          </span>
          {identity.is_service_account && (
            <Badge variant="outline" className="shrink-0 text-[10px]">
              bot
            </Badge>
          )}
        </div>
        <div className="flex items-center gap-2 text-xs text-muted-foreground">
          {identity.username && <span>@{identity.username}</span>}
          {identity.email && <span>{identity.email}</span>}
        </div>
      </div>
      {selected && (
        <div className="h-4 w-4 shrink-0 rounded-full bg-primary" />
      )}
    </button>
  );
}
