"use client";

import { useState, useMemo, useCallback } from "react";
import { useParams } from "next/navigation";
import Link from "next/link";
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
  Select,
  SelectContent,
  SelectItem,
  SelectTrigger,
  SelectValue,
} from "@/components/ui/select";
import { Separator } from "@/components/ui/separator";
import { LoadingState } from "@/components/states/loading";
import { ErrorState } from "@/components/states/error";
import { EmptyState } from "@/components/states/empty";
import {
  usePerson,
  usePersonIdentities,
  usePersonActivity,
  useUnlinkIdentity,
} from "@/features/people/hooks/use-people";
import { LinkIdentityDialog } from "@/features/people/components/link-identity-dialog";
import { useTranslation, formatShortDate, formatDateTime } from "@/i18n";
import type { ActivityFilter } from "@/lib/api/types";
import {
  ArrowLeft,
  Plus,
  X,
  GitMerge,
  Link2,
  Mail,
  Users,
  Briefcase,
  Calendar,
  Activity,
} from "lucide-react";

export default function Person360Page() {
  const params = useParams<{ id: string }>();
  const personId = params.id;
  const { t, locale } = useTranslation();

  const [linkDialogOpen, setLinkDialogOpen] = useState(false);
  const [periodFilter, setPeriodFilter] = useState<string>("all");
  const [sourceFilter, setSourceFilter] = useState<string>("all");
  const [typeFilter, setTypeFilter] = useState<string>("all");

  const activityFilter: ActivityFilter = useMemo(
    () => ({
      period: periodFilter === "all" ? undefined : periodFilter,
      source: sourceFilter === "all" ? undefined : sourceFilter,
      type: typeFilter === "all" ? undefined : typeFilter,
      limit: 50,
    }),
    [periodFilter, sourceFilter, typeFilter],
  );

  const {
    data: person,
    isLoading: personLoading,
    isError: personError,
    error: personErr,
    refetch: refetchPerson,
  } = usePerson(personId);

  const {
    data: identities,
    isLoading: identitiesLoading,
  } = usePersonIdentities(personId);

  const {
    data: activity,
    isLoading: activityLoading,
  } = usePersonActivity(personId, activityFilter);

  const unlinkMutation = useUnlinkIdentity(personId);

  const handleUnlink = useCallback(
    (identityId: string) => {
      unlinkMutation.mutate(identityId);
    },
    [unlinkMutation],
  );

  // Loading state
  if (personLoading) {
    return (
      <div className="space-y-6">
        <LoadingState rows={6} />
      </div>
    );
  }

  // Error state
  if (personError || !person) {
    return (
      <div className="space-y-6">
        <Link href="/team/people" className="inline-flex items-center gap-1 text-sm text-muted-foreground hover:text-foreground">
          <ArrowLeft className="h-4 w-4" />
          {t("person360.back")}
        </Link>
        <ErrorState
          message={
            personErr instanceof Error
              ? personErr.message
              : t("person360.failedToLoad")
          }
          onRetry={() => refetchPerson()}
        />
      </div>
    );
  }

  return (
    <div className="space-y-6">
      {/* Back link */}
      <Link
        href="/team/people"
        className="inline-flex items-center gap-1 text-sm text-muted-foreground hover:text-foreground"
      >
        <ArrowLeft className="h-4 w-4" />
        {t("person360.back")}
      </Link>

      {/* Person header */}
      <div className="flex items-start justify-between">
        <div className="flex items-center gap-4">
          {/* Avatar placeholder */}
          <div className="flex h-16 w-16 items-center justify-center rounded-full bg-muted text-2xl font-semibold">
            {person.display_name.charAt(0).toUpperCase()}
          </div>
          <div>
            <h1 className="text-2xl font-semibold">{person.display_name}</h1>
            <div className="mt-1 flex flex-wrap items-center gap-3 text-sm text-muted-foreground">
              {person.team && (
                <span className="inline-flex items-center gap-1">
                  <Users className="h-3.5 w-3.5" />
                  {person.team}
                </span>
              )}
              {person.role && (
                <span className="inline-flex items-center gap-1">
                  <Briefcase className="h-3.5 w-3.5" />
                  {person.role}
                </span>
              )}
              {person.primary_email && (
                <span className="inline-flex items-center gap-1">
                  <Mail className="h-3.5 w-3.5" />
                  {person.primary_email}
                </span>
              )}
              <span className="inline-flex items-center gap-1">
                <Calendar className="h-3.5 w-3.5" />
                {t("person360.memberSince")}{" "}
                {formatShortDate(person.created_at.split("T")[0] ?? person.created_at, locale)}
              </span>
            </div>
          </div>
        </div>
        <Badge variant={person.status === "active" ? "secondary" : "outline"}>
          {person.status === "active" ? t("people.active") : t("people.inactive")}
        </Badge>
      </div>

      <Separator />

      {/* Two-column layout */}
      <div className="grid gap-6 lg:grid-cols-3">
        {/* Left: Identities + Stats */}
        <div className="space-y-6 lg:col-span-1">
          {/* Identities Card */}
          <Card>
            <CardHeader className="flex flex-row items-center justify-between pb-3">
              <div>
                <CardTitle className="text-base">{t("person360.identities")}</CardTitle>
                <CardDescription>
                  {person.identity_count} {t("people.colIdentities").toLowerCase()}
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
                <div className="space-y-3">
                  {identities.data.map((identity) => (
                    <div
                      key={identity.link_id}
                      className="flex items-center justify-between rounded-md border px-3 py-2"
                    >
                      <div className="min-w-0 flex-1">
                        <div className="flex items-center gap-2">
                          <SourceBadge source={identity.source} />
                          <span className="truncate text-sm font-medium">
                            {identity.display_name ?? identity.username ?? identity.email ?? identity.identity_id}
                          </span>
                        </div>
                        {identity.username && (
                          <p className="mt-0.5 text-xs text-muted-foreground truncate">
                            @{identity.username}
                          </p>
                        )}
                      </div>
                      <Button
                        size="icon"
                        variant="ghost"
                        className="h-7 w-7 shrink-0"
                        onClick={() => handleUnlink(identity.identity_id)}
                        disabled={unlinkMutation.isPending}
                        title={t("person360.unlink")}
                      >
                        <X className="h-3.5 w-3.5" />
                      </Button>
                    </div>
                  ))}
                </div>
              )}
            </CardContent>
          </Card>

          {/* Stats Card */}
          <Card>
            <CardHeader className="pb-3">
              <CardTitle className="text-base">{t("person360.stats")}</CardTitle>
            </CardHeader>
            <CardContent>
              <div className="space-y-2">
                <StatRow
                  icon={<Link2 className="h-4 w-4" />}
                  label={t("people.colIdentities")}
                  value={String(person.identity_count)}
                />
                <StatRow
                  icon={<Activity className="h-4 w-4" />}
                  label={t("person360.totalActivity")}
                  value={String(activity?.total ?? 0)}
                />
              </div>
            </CardContent>
          </Card>
        </div>

        {/* Right: Activity Timeline */}
        <div className="lg:col-span-2">
          <Card>
            <CardHeader className="pb-3">
              <CardTitle className="text-base">{t("person360.activity")}</CardTitle>
              {/* Filters */}
              <div className="flex flex-wrap items-center gap-2 pt-2">
                <Select value={periodFilter} onValueChange={setPeriodFilter}>
                  <SelectTrigger className="w-[130px]">
                    <SelectValue />
                  </SelectTrigger>
                  <SelectContent>
                    <SelectItem value="all">{t("person360.periodAll")}</SelectItem>
                    <SelectItem value="7d">{t("person360.period7d")}</SelectItem>
                    <SelectItem value="30d">{t("person360.period30d")}</SelectItem>
                    <SelectItem value="90d">{t("person360.period90d")}</SelectItem>
                  </SelectContent>
                </Select>

                <Select value={sourceFilter} onValueChange={setSourceFilter}>
                  <SelectTrigger className="w-[140px]">
                    <SelectValue />
                  </SelectTrigger>
                  <SelectContent>
                    <SelectItem value="all">{t("person360.sourceAll")}</SelectItem>
                    <SelectItem value="gitlab">{t("person360.sourceGitlab")}</SelectItem>
                    <SelectItem value="jira">{t("person360.sourceJira")}</SelectItem>
                    <SelectItem value="identity">{t("person360.sourceIdentity")}</SelectItem>
                  </SelectContent>
                </Select>

                <Select value={typeFilter} onValueChange={setTypeFilter}>
                  <SelectTrigger className="w-[160px]">
                    <SelectValue />
                  </SelectTrigger>
                  <SelectContent>
                    <SelectItem value="all">{t("person360.typeAll")}</SelectItem>
                    <SelectItem value="merge_request">{t("person360.typeMr")}</SelectItem>
                    <SelectItem value="issue">{t("person360.typeIssue")}</SelectItem>
                    <SelectItem value="identity_event">{t("person360.typeIdentityEvent")}</SelectItem>
                  </SelectContent>
                </Select>
              </div>
            </CardHeader>
            <CardContent>
              {activityLoading ? (
                <LoadingState rows={5} />
              ) : !activity?.data.length ? (
                <EmptyState
                  title={t("person360.noActivity")}
                  description={t("person360.noActivityDesc")}
                />
              ) : (
                <div className="space-y-1">
                  {activity.data.map((item) => (
                    <ActivityRow key={item.id} item={item} locale={locale} />
                  ))}
                </div>
              )}
            </CardContent>
          </Card>
        </div>
      </div>

      {/* Link Identity Dialog */}
      <LinkIdentityDialog
        personId={personId}
        open={linkDialogOpen}
        onOpenChange={setLinkDialogOpen}
      />
    </div>
  );
}

// ── Helpers ──────────────────────────────────────────────────

function SourceBadge({ source }: { source: string }) {
  const colors: Record<string, string> = {
    gitlab: "bg-orange-100 text-orange-800 dark:bg-orange-900 dark:text-orange-200",
    jira: "bg-blue-100 text-blue-800 dark:bg-blue-900 dark:text-blue-200",
    confluence: "bg-teal-100 text-teal-800 dark:bg-teal-900 dark:text-teal-200",
    git: "bg-gray-100 text-gray-800 dark:bg-gray-800 dark:text-gray-200",
  };
  return (
    <span
      className={`inline-flex items-center rounded px-1.5 py-0.5 text-xs font-medium ${
        colors[source] ?? "bg-muted text-muted-foreground"
      }`}
    >
      {source}
    </span>
  );
}

function StatRow({
  icon,
  label,
  value,
}: {
  icon: React.ReactNode;
  label: string;
  value: string;
}) {
  return (
    <div className="flex items-center justify-between">
      <span className="inline-flex items-center gap-2 text-sm text-muted-foreground">
        {icon}
        {label}
      </span>
      <span className="text-sm font-medium">{value}</span>
    </div>
  );
}

function ActivityRow({
  item,
  locale,
}: {
  item: { id: string; source: string; type: string; title: string; url: string | null; timestamp: string; metadata: Record<string, unknown> };
  locale: "en" | "ru";
}) {
  const icon =
    item.source === "gitlab" ? (
      <GitMerge className="h-4 w-4 text-orange-500" />
    ) : item.source === "jira" ? (
      <Briefcase className="h-4 w-4 text-blue-500" />
    ) : (
      <Link2 className="h-4 w-4 text-muted-foreground" />
    );

  return (
    <div className="flex items-start gap-3 rounded-md px-2 py-2 hover:bg-muted/50">
      <div className="mt-0.5 shrink-0">{icon}</div>
      <div className="min-w-0 flex-1">
        <div className="flex items-center gap-2">
          <SourceBadge source={item.source} />
          {item.url ? (
            <a
              href={item.url}
              target="_blank"
              rel="noopener noreferrer"
              className="truncate text-sm font-medium hover:underline"
            >
              {item.title}
            </a>
          ) : (
            <span className="truncate text-sm font-medium">{item.title}</span>
          )}
        </div>
        <p className="mt-0.5 text-xs text-muted-foreground">
          {formatDateTime(item.timestamp, locale)}
          {typeof item.metadata?.state === "string" && (
            <> &middot; {item.metadata.state}</>
          )}
        </p>
      </div>
    </div>
  );
}
