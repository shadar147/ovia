"use client";

import { useState, useMemo, useCallback } from "react";
import { Button } from "@/components/ui/button";
import { Badge } from "@/components/ui/badge";
import {
  Card,
  CardContent,
  CardHeader,
  CardTitle,
  CardDescription,
} from "@/components/ui/card";
import {
  Dialog,
  DialogContent,
  DialogHeader,
  DialogTitle,
  DialogDescription,
  DialogFooter,
} from "@/components/ui/dialog";
import { LoadingState } from "@/components/states/loading";
import { EmptyState } from "@/components/states/empty";
import {
  usePersonIdentities,
  useUnlinkIdentity,
  useLinkIdentity,
} from "@/features/people/hooks/use-people";
import { LinkIdentityDialog } from "./link-identity-dialog";
import { useTranslation, formatShortDate } from "@/i18n";
import type { LinkedIdentityResponse } from "@/lib/api/types";
import {
  Plus,
  X,
  GitMerge,
  Briefcase,
  FileText,
  GitCommit,
  Shield,
  ShieldCheck,
  ShieldAlert,
} from "lucide-react";

interface IdentityLinkPanelProps {
  personId: string;
  identityCount: number;
}

const SOURCE_CONFIG: Record<
  string,
  { icon: React.ElementType; color: string; label: string }
> = {
  gitlab: {
    icon: GitMerge,
    color: "bg-orange-100 text-orange-800 dark:bg-orange-900 dark:text-orange-200",
    label: "GitLab",
  },
  jira: {
    icon: Briefcase,
    color: "bg-blue-100 text-blue-800 dark:bg-blue-900 dark:text-blue-200",
    label: "Jira",
  },
  confluence: {
    icon: FileText,
    color: "bg-teal-100 text-teal-800 dark:bg-teal-900 dark:text-teal-200",
    label: "Confluence",
  },
  git_commit_author: {
    icon: GitCommit,
    color: "bg-gray-100 text-gray-800 dark:bg-gray-800 dark:text-gray-200",
    label: "Git",
  },
  git_commit_committer: {
    icon: GitCommit,
    color: "bg-gray-100 text-gray-800 dark:bg-gray-800 dark:text-gray-200",
    label: "Git",
  },
  git_config_snapshot: {
    icon: GitCommit,
    color: "bg-gray-100 text-gray-800 dark:bg-gray-800 dark:text-gray-200",
    label: "Git",
  },
};

const STATUS_CONFIG: Record<string, { icon: React.ElementType; variant: "secondary" | "outline" | "destructive" }> = {
  verified: { icon: ShieldCheck, variant: "secondary" },
  auto: { icon: Shield, variant: "outline" },
  conflict: { icon: ShieldAlert, variant: "destructive" },
};

export function IdentityLinkPanel({ personId, identityCount }: IdentityLinkPanelProps) {
  const { t, locale } = useTranslation();
  const [linkDialogOpen, setLinkDialogOpen] = useState(false);
  const [unlinkTarget, setUnlinkTarget] = useState<LinkedIdentityResponse | null>(null);

  const { data: identities, isLoading: identitiesLoading } =
    usePersonIdentities(personId);
  const unlinkMutation = useUnlinkIdentity(personId);

  // Group identities by source
  const groupedIdentities = useMemo(() => {
    if (!identities?.data.length) return new Map<string, LinkedIdentityResponse[]>();
    const groups = new Map<string, LinkedIdentityResponse[]>();
    for (const identity of identities.data) {
      const source = identity.source;
      if (!groups.has(source)) groups.set(source, []);
      groups.get(source)!.push(identity);
    }
    return groups;
  }, [identities]);

  const handleUnlinkConfirm = useCallback(() => {
    if (!unlinkTarget) return;
    unlinkMutation.mutate(unlinkTarget.identity_id, {
      onSuccess: () => setUnlinkTarget(null),
    });
  }, [unlinkTarget, unlinkMutation]);

  return (
    <>
      <Card>
        <CardHeader className="flex flex-row items-center justify-between pb-3">
          <div>
            <CardTitle className="text-base">{t("person360.identities")}</CardTitle>
            <CardDescription>
              {identityCount} {t("people.colIdentities").toLowerCase()}
            </CardDescription>
          </div>
          <Button
            size="xs"
            variant="outline"
            onClick={() => setLinkDialogOpen(true)}
          >
            <Plus className="mr-1 h-3 w-3" />
            {t("person360.linkIdentity")}
          </Button>
        </CardHeader>
        <CardContent>
          {identitiesLoading ? (
            <LoadingState rows={3} />
          ) : !identities?.data.length ? (
            <EmptyState
              title={t("person360.noIdentities")}
              description={t("person360.noIdentitiesDesc")}
            />
          ) : (
            <div className="space-y-4">
              {Array.from(groupedIdentities.entries()).map(([source, items]) => {
                const config = SOURCE_CONFIG[source];
                const Icon = config?.icon ?? Shield;
                const label = config?.label ?? source;

                return (
                  <div key={source}>
                    {/* Source group header */}
                    <div className="mb-2 flex items-center gap-2">
                      <span
                        className={`inline-flex items-center gap-1 rounded px-1.5 py-0.5 text-xs font-medium ${
                          config?.color ?? "bg-muted text-muted-foreground"
                        }`}
                      >
                        <Icon className="h-3 w-3" />
                        {label}
                      </span>
                      <span className="text-xs text-muted-foreground">
                        ({items.length})
                      </span>
                    </div>

                    {/* Identity rows */}
                    <div className="space-y-1.5">
                      {items.map((identity) => {
                        const statusConfig = STATUS_CONFIG[identity.status];
                        const StatusIcon = statusConfig?.icon ?? Shield;

                        return (
                          <div
                            key={identity.link_id}
                            className="flex items-center justify-between rounded-md border px-3 py-2"
                          >
                            <div className="min-w-0 flex-1">
                              <div className="flex items-center gap-2">
                                <span className="truncate text-sm font-medium">
                                  {identity.display_name ??
                                    identity.username ??
                                    identity.email ??
                                    identity.identity_id}
                                </span>
                                <Badge
                                  variant={statusConfig?.variant ?? "outline"}
                                  className="shrink-0 text-[10px] px-1.5 py-0"
                                >
                                  <StatusIcon className="mr-0.5 h-2.5 w-2.5" />
                                  {t(`status.${identity.status}` as Parameters<typeof t>[0])}
                                </Badge>
                                {identity.confidence < 1 && (
                                  <span className="shrink-0 text-[10px] text-muted-foreground">
                                    {Math.round(identity.confidence * 100)}%
                                  </span>
                                )}
                              </div>
                              <div className="mt-0.5 flex items-center gap-2 text-xs text-muted-foreground">
                                {identity.username && (
                                  <span className="truncate">@{identity.username}</span>
                                )}
                                {identity.email && (
                                  <span className="truncate">{identity.email}</span>
                                )}
                                <span className="shrink-0">
                                  {formatShortDate(
                                    identity.linked_at.split("T")[0] ?? identity.linked_at,
                                    locale,
                                  )}
                                </span>
                              </div>
                            </div>
                            <Button
                              size="icon"
                              variant="ghost"
                              className="h-7 w-7 shrink-0"
                              onClick={() => setUnlinkTarget(identity)}
                              disabled={unlinkMutation.isPending}
                              title={t("person360.unlink")}
                            >
                              <X className="h-3.5 w-3.5" />
                            </Button>
                          </div>
                        );
                      })}
                    </div>
                  </div>
                );
              })}
            </div>
          )}
        </CardContent>
      </Card>

      {/* Link Identity Dialog */}
      <LinkIdentityDialog
        personId={personId}
        open={linkDialogOpen}
        onOpenChange={setLinkDialogOpen}
      />

      {/* Unlink Confirmation Dialog */}
      <Dialog open={!!unlinkTarget} onOpenChange={(open) => !open && setUnlinkTarget(null)}>
        <DialogContent>
          <DialogHeader>
            <DialogTitle>{t("person360.unlinkConfirm")}</DialogTitle>
            <DialogDescription>{t("person360.unlinkConfirmDesc")}</DialogDescription>
          </DialogHeader>

          {unlinkTarget && (
            <div className="rounded-md border px-3 py-2">
              <div className="flex items-center gap-2">
                <SourceBadge source={unlinkTarget.source} />
                <span className="text-sm font-medium">
                  {unlinkTarget.display_name ??
                    unlinkTarget.username ??
                    unlinkTarget.email ??
                    unlinkTarget.identity_id}
                </span>
              </div>
              {unlinkTarget.username && (
                <p className="mt-0.5 text-xs text-muted-foreground">
                  @{unlinkTarget.username}
                </p>
              )}
            </div>
          )}

          <DialogFooter>
            <Button
              type="button"
              variant="outline"
              onClick={() => setUnlinkTarget(null)}
            >
              {t("people.cancel")}
            </Button>
            <Button
              variant="destructive"
              onClick={handleUnlinkConfirm}
              disabled={unlinkMutation.isPending}
            >
              {unlinkMutation.isPending
                ? t("person360.unlinking")
                : t("person360.unlink")}
            </Button>
          </DialogFooter>
        </DialogContent>
      </Dialog>
    </>
  );
}

function SourceBadge({ source }: { source: string }) {
  const config = SOURCE_CONFIG[source];
  const Icon = config?.icon ?? Shield;
  return (
    <span
      className={`inline-flex items-center gap-1 rounded px-1.5 py-0.5 text-xs font-medium ${
        config?.color ?? "bg-muted text-muted-foreground"
      }`}
    >
      <Icon className="h-3 w-3" />
      {config?.label ?? source}
    </span>
  );
}
