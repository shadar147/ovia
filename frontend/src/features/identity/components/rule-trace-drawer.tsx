"use client";

import { Sheet, SheetContent, SheetHeader, SheetTitle, SheetDescription } from "@/components/ui/sheet";
import { Badge } from "@/components/ui/badge";
import { Separator } from "@/components/ui/separator";
import { Button } from "@/components/ui/button";
import { StatusBadge, ConfidenceBadge } from "./status-badge";
import type { PersonIdentityLink } from "@/lib/api/types";
import { useTranslation, formatDateTime } from "@/i18n";

interface RuleTraceDrawerProps {
  link: PersonIdentityLink | null;
  open: boolean;
  onClose: () => void;
  onConfirm: (linkId: string) => void;
  onSplit: (linkId: string) => void;
  onRemap: (linkId: string) => void;
  isPending: boolean;
}

export function RuleTraceDrawer({
  link,
  open,
  onClose,
  onConfirm,
  onSplit,
  onRemap,
  isPending,
}: RuleTraceDrawerProps) {
  const { t, locale } = useTranslation();

  if (!link) return null;

  const trace = link.rule_trace;
  const canAct = link.status === "auto" || link.status === "conflict";

  return (
    <Sheet open={open} onOpenChange={(v) => !v && onClose()}>
      <SheetContent className="w-full sm:max-w-lg">
        <SheetHeader>
          <SheetTitle>{t("identity.drawerTitle")}</SheetTitle>
          <SheetDescription>
            {link.person?.display_name ?? t("identity.unknown")} → {link.identity?.display_name ?? link.identity?.username ?? t("identity.unknown")}
          </SheetDescription>
        </SheetHeader>

        <div className="px-4 pb-4 space-y-6 flex-1 overflow-y-auto">
          {/* Person */}
          <section>
            <h3 className="text-sm font-medium text-muted-foreground mb-2">{t("identity.person")}</h3>
            <div className="space-y-1">
              <p className="font-medium">{link.person?.display_name ?? t("identity.unknown")}</p>
              <p className="text-sm text-muted-foreground">{link.person?.primary_email ?? "—"}</p>
              {link.person?.team && (
                <Badge variant="outline">{link.person.team}</Badge>
              )}
            </div>
          </section>

          <Separator />

          {/* Identity */}
          <section>
            <h3 className="text-sm font-medium text-muted-foreground mb-2">{t("identity.identity")}</h3>
            <div className="space-y-1">
              <p className="font-medium">
                {link.identity?.display_name ?? link.identity?.username ?? t("identity.unknown")}
              </p>
              <p className="text-sm text-muted-foreground">{link.identity?.email ?? "—"}</p>
              <div className="flex gap-2">
                <Badge variant="outline">{link.identity?.source ?? "?"}</Badge>
                {link.identity?.is_service_account && (
                  <Badge variant="destructive">{t("identity.serviceAccount")}</Badge>
                )}
              </div>
            </div>
          </section>

          <Separator />

          {/* Status + Confidence */}
          <section className="flex items-center gap-3">
            <StatusBadge status={link.status} />
            <ConfidenceBadge confidence={link.confidence} />
            {link.verified_by && (
              <span className="text-xs text-muted-foreground">by {link.verified_by}</span>
            )}
          </section>

          <Separator />

          {/* Rule trace */}
          {trace && (
            <section>
              <h3 className="text-sm font-medium text-muted-foreground mb-2">{t("identity.matchRationale")}</h3>
              <div className="space-y-2">
                {trace.scorers.map((s) => (
                  <div key={s.rule} className="flex items-center justify-between text-sm">
                    <span className="font-mono">{s.rule}</span>
                    <div className="flex items-center gap-2">
                      <span className="text-muted-foreground">
                        {(s.score * 100).toFixed(0)}% x {s.weight.toFixed(2)}
                      </span>
                      <span className="font-medium w-12 text-right">
                        {(s.weighted_score * 100).toFixed(1)}
                      </span>
                    </div>
                  </div>
                ))}
                <Separator />
                <div className="flex items-center justify-between text-sm font-medium">
                  <span>{t("identity.totalConfidence")}</span>
                  <span>{(trace.confidence * 100).toFixed(1)}%</span>
                </div>
              </div>
            </section>
          )}

          <Separator />

          {/* Audit info */}
          <section>
            <h3 className="text-sm font-medium text-muted-foreground mb-2">{t("identity.audit")}</h3>
            <div className="text-sm space-y-1">
              <p>{t("identity.created", { date: formatDateTime(link.created_at, locale) })}</p>
              <p>{t("identity.updated", { date: formatDateTime(link.updated_at, locale) })}</p>
            </div>
          </section>

          {/* Actions */}
          {canAct && (
            <>
              <Separator />
              <div className="flex gap-2">
                <Button size="sm" onClick={() => onConfirm(link.id)} disabled={isPending}>
                  {t("identity.confirm")}
                </Button>
                <Button size="sm" variant="outline" onClick={() => onRemap(link.id)} disabled={isPending}>
                  {t("identity.remap")}
                </Button>
                <Button size="sm" variant="destructive" onClick={() => onSplit(link.id)} disabled={isPending}>
                  {t("identity.split")}
                </Button>
              </div>
            </>
          )}
        </div>
      </SheetContent>
    </Sheet>
  );
}
