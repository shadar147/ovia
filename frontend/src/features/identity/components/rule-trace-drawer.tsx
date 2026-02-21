"use client";

import { Sheet, SheetContent, SheetHeader, SheetTitle } from "@/components/ui/sheet";
import { Badge } from "@/components/ui/badge";
import { Separator } from "@/components/ui/separator";
import { Button } from "@/components/ui/button";
import { StatusBadge, ConfidenceBadge } from "./status-badge";
import type { PersonIdentityLink } from "@/lib/api/types";

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
  if (!link) return null;

  const trace = link.rule_trace;
  const canAct = link.status === "auto" || link.status === "conflict";

  return (
    <Sheet open={open} onOpenChange={(v) => !v && onClose()}>
      <SheetContent className="w-full sm:max-w-lg overflow-y-auto">
        <SheetHeader>
          <SheetTitle>Mapping Details</SheetTitle>
        </SheetHeader>

        <div className="mt-6 space-y-6">
          {/* Person */}
          <section>
            <h3 className="text-sm font-medium text-muted-foreground mb-2">Person</h3>
            <div className="space-y-1">
              <p className="font-medium">{link.person?.display_name ?? "Unknown"}</p>
              <p className="text-sm text-muted-foreground">{link.person?.primary_email ?? "—"}</p>
              {link.person?.team && (
                <Badge variant="outline">{link.person.team}</Badge>
              )}
            </div>
          </section>

          <Separator />

          {/* Identity */}
          <section>
            <h3 className="text-sm font-medium text-muted-foreground mb-2">Identity</h3>
            <div className="space-y-1">
              <p className="font-medium">
                {link.identity?.display_name ?? link.identity?.username ?? "Unknown"}
              </p>
              <p className="text-sm text-muted-foreground">{link.identity?.email ?? "—"}</p>
              <div className="flex gap-2">
                <Badge variant="outline">{link.identity?.source ?? "?"}</Badge>
                {link.identity?.is_service_account && (
                  <Badge variant="destructive">Service Account</Badge>
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
              <h3 className="text-sm font-medium text-muted-foreground mb-2">Match Rationale</h3>
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
                  <span>Total confidence</span>
                  <span>{(trace.confidence * 100).toFixed(1)}%</span>
                </div>
              </div>
            </section>
          )}

          <Separator />

          {/* Audit info */}
          <section>
            <h3 className="text-sm font-medium text-muted-foreground mb-2">Audit</h3>
            <div className="text-sm space-y-1">
              <p>Created: {new Date(link.created_at).toLocaleString()}</p>
              <p>Updated: {new Date(link.updated_at).toLocaleString()}</p>
            </div>
          </section>

          {/* Actions */}
          {canAct && (
            <>
              <Separator />
              <div className="flex gap-2">
                <Button size="sm" onClick={() => onConfirm(link.id)} disabled={isPending}>
                  Confirm
                </Button>
                <Button size="sm" variant="outline" onClick={() => onRemap(link.id)} disabled={isPending}>
                  Remap
                </Button>
                <Button size="sm" variant="destructive" onClick={() => onSplit(link.id)} disabled={isPending}>
                  Split
                </Button>
              </div>
            </>
          )}
        </div>
      </SheetContent>
    </Sheet>
  );
}
