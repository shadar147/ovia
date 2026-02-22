"use client";

import { useState } from "react";
import {
  Table,
  TableBody,
  TableCell,
  TableHead,
  TableHeader,
  TableRow,
} from "@/components/ui/table";
import { Badge } from "@/components/ui/badge";
import { Button } from "@/components/ui/button";
import type { RiskItem } from "@/lib/api/types";
import { ExternalLink, ChevronLeft, ChevronRight } from "lucide-react";
import { cn } from "@/lib/utils";
import { useTranslation } from "@/i18n";

const PAGE_SIZE = 20;

interface RiskTableProps {
  risks: RiskItem[];
}

const entityColors: Record<string, string> = {
  pull_request: "bg-blue-100 text-blue-800",
  issue: "bg-orange-100 text-orange-800",
  pipeline: "bg-purple-100 text-purple-800",
};

function ageColor(days: number): string {
  if (days >= 14) return "text-red-600 font-semibold";
  if (days >= 7) return "text-yellow-600";
  return "text-muted-foreground";
}

function entityLabel(type: string): string {
  return type.replace(/_/g, " ").replace(/\b\w/g, (c) => c.toUpperCase());
}

export function RiskTable({ risks }: RiskTableProps) {
  const { t } = useTranslation();
  const [page, setPage] = useState(0);

  if (risks.length === 0) {
    return (
      <p className="py-8 text-center text-sm text-muted-foreground">
        {t("risk.noRisks")}
      </p>
    );
  }

  const totalPages = Math.ceil(risks.length / PAGE_SIZE);
  const pagedRisks = risks.slice(page * PAGE_SIZE, (page + 1) * PAGE_SIZE);

  return (
    <div>
      <Table>
        <TableHeader>
          <TableRow>
            <TableHead>{t("risk.type")}</TableHead>
            <TableHead>{t("risk.title")}</TableHead>
            <TableHead>{t("risk.owner")}</TableHead>
            <TableHead className="text-right">{t("risk.age")}</TableHead>
            <TableHead>{t("risk.status")}</TableHead>
            <TableHead className="w-10" />
          </TableRow>
        </TableHeader>
        <TableBody>
          {pagedRisks.map((risk) => (
            <TableRow key={risk.id}>
              <TableCell>
                <Badge
                  variant="secondary"
                  className={cn("text-xs", entityColors[risk.entity_type])}
                >
                  {entityLabel(risk.entity_type)}
                </Badge>
              </TableCell>
              <TableCell className="max-w-[300px] truncate font-medium">
                {risk.title}
              </TableCell>
              <TableCell className="text-muted-foreground">
                {risk.owner ?? <span className="italic">{t("risk.unassigned")}</span>}
              </TableCell>
              <TableCell className={cn("text-right", ageColor(risk.age_days))}>
                {risk.age_days}d
              </TableCell>
              <TableCell>
                <Badge variant="outline" className="text-xs capitalize">
                  {risk.status.replace(/_/g, " ")}
                </Badge>
              </TableCell>
              <TableCell>
                {risk.source_url && (
                  <a
                    href={risk.source_url}
                    target="_blank"
                    rel="noopener noreferrer"
                    className="text-muted-foreground hover:text-foreground"
                  >
                    <ExternalLink className="h-4 w-4" />
                  </a>
                )}
              </TableCell>
            </TableRow>
          ))}
        </TableBody>
      </Table>
      {totalPages > 1 && (
        <div className="flex items-center justify-between border-t px-2 py-3">
          <span className="text-xs text-muted-foreground">
            {t("risk.pageInfo", {
              from: String(page * PAGE_SIZE + 1),
              to: String(Math.min((page + 1) * PAGE_SIZE, risks.length)),
              total: String(risks.length),
            })}
          </span>
          <div className="flex gap-1">
            <Button
              variant="outline"
              size="sm"
              disabled={page === 0}
              onClick={() => setPage((p) => p - 1)}
            >
              <ChevronLeft className="h-4 w-4" />
            </Button>
            <Button
              variant="outline"
              size="sm"
              disabled={page >= totalPages - 1}
              onClick={() => setPage((p) => p + 1)}
            >
              <ChevronRight className="h-4 w-4" />
            </Button>
          </div>
        </div>
      )}
    </div>
  );
}
