"use client";

import {
  useReactTable,
  getCoreRowModel,
  getSortedRowModel,
  flexRender,
  type ColumnDef,
  type SortingState,
} from "@tanstack/react-table";
import { useState, useMemo } from "react";
import {
  Table,
  TableBody,
  TableCell,
  TableHead,
  TableHeader,
  TableRow,
} from "@/components/ui/table";
import { Button } from "@/components/ui/button";
import { Badge } from "@/components/ui/badge";
import { ArrowUpDown } from "lucide-react";
import type { PersonResponse } from "@/lib/api/types";
import { useTranslation, formatShortDate } from "@/i18n";
import type { MessageKey } from "@/i18n";

interface PeopleTableProps {
  data: PersonResponse[];
  total: number;
  page: number;
  pageSize: number;
  onPageChange: (page: number) => void;
}

function SortableHeader({
  column,
  labelKey,
}: {
  column: {
    toggleSorting: (desc: boolean) => void;
    getIsSorted: () => false | "asc" | "desc";
  };
  labelKey: MessageKey;
}) {
  const { t } = useTranslation();
  return (
    <Button
      variant="ghost"
      size="sm"
      className="-ml-3"
      onClick={() => column.toggleSorting(column.getIsSorted() === "asc")}
    >
      {t(labelKey)}
      <ArrowUpDown className="ml-1 h-3 w-3" />
    </Button>
  );
}

function StatusBadge({ status }: { status: string }) {
  const { t } = useTranslation();
  const variant = status === "active" ? "secondary" : "outline";
  const label =
    status === "active" ? t("people.active") : t("people.inactive");
  return <Badge variant={variant}>{label}</Badge>;
}

function IdentityCountBadge({ count }: { count: number }) {
  if (count === 0) return <span className="text-sm text-muted-foreground">—</span>;
  return <Badge variant="outline">{count}</Badge>;
}

export function PeopleTable({
  data,
  total,
  page,
  pageSize,
  onPageChange,
}: PeopleTableProps) {
  const [sorting, setSorting] = useState<SortingState>([]);
  const { t, locale } = useTranslation();

  const columns: ColumnDef<PersonResponse>[] = useMemo(
    () => [
      {
        accessorKey: "display_name",
        header: ({ column }) => (
          <SortableHeader column={column} labelKey="people.colName" />
        ),
        cell: ({ row }) => (
          <div>
            <p className="font-medium">{row.original.display_name}</p>
            {row.original.primary_email && (
              <p className="text-xs text-muted-foreground">
                {row.original.primary_email}
              </p>
            )}
          </div>
        ),
      },
      {
        accessorKey: "team",
        header: ({ column }) => (
          <SortableHeader column={column} labelKey="people.colTeam" />
        ),
        cell: ({ row }) => (
          <span className="text-sm">
            {row.original.team ?? "—"}
          </span>
        ),
      },
      {
        accessorKey: "role",
        header: () => t("people.colRole"),
        cell: ({ row }) => (
          <span className="text-sm">
            {row.original.role ?? "—"}
          </span>
        ),
        enableSorting: false,
      },
      {
        accessorKey: "identity_count",
        header: ({ column }) => (
          <SortableHeader column={column} labelKey="people.colIdentities" />
        ),
        cell: ({ row }) => (
          <IdentityCountBadge count={row.original.identity_count} />
        ),
      },
      {
        accessorKey: "status",
        header: () => t("people.colStatus"),
        cell: ({ row }) => <StatusBadge status={row.original.status} />,
        enableSorting: false,
      },
      {
        accessorKey: "created_at",
        header: ({ column }) => (
          <SortableHeader column={column} labelKey="people.colCreated" />
        ),
        cell: ({ row }) => (
          <span className="text-sm text-muted-foreground">
            {formatShortDate(
              row.original.created_at.split("T")[0] ?? row.original.created_at,
              locale,
            )}
          </span>
        ),
      },
    ],
    [t, locale],
  );

  const table = useReactTable({
    data,
    columns,
    getCoreRowModel: getCoreRowModel(),
    getSortedRowModel: getSortedRowModel(),
    onSortingChange: setSorting,
    state: { sorting },
  });

  const totalPages = Math.ceil(total / pageSize);
  const from = total === 0 ? 0 : page * pageSize + 1;
  const to = Math.min((page + 1) * pageSize, total);

  return (
    <div>
      <div className="rounded-md border">
        <Table>
          <TableHeader>
            {table.getHeaderGroups().map((hg) => (
              <TableRow key={hg.id}>
                {hg.headers.map((h) => (
                  <TableHead key={h.id} className="h-12">
                    {h.isPlaceholder
                      ? null
                      : flexRender(h.column.columnDef.header, h.getContext())}
                  </TableHead>
                ))}
              </TableRow>
            ))}
          </TableHeader>
          <TableBody>
            {table.getRowModel().rows.length === 0 ? (
              <TableRow>
                <TableCell
                  colSpan={columns.length}
                  className="h-24 text-center"
                >
                  {t("people.noResults")}
                </TableCell>
              </TableRow>
            ) : (
              table.getRowModel().rows.map((row) => (
                <TableRow key={row.id}>
                  {row.getVisibleCells().map((cell) => (
                    <TableCell key={cell.id} className="h-[52px]">
                      {flexRender(
                        cell.column.columnDef.cell,
                        cell.getContext(),
                      )}
                    </TableCell>
                  ))}
                </TableRow>
              ))
            )}
          </TableBody>
        </Table>
      </div>

      {/* Server-side pagination */}
      <div className="flex items-center justify-between py-4">
        <span className="text-sm text-muted-foreground">
          {total > 0
            ? t("people.pageInfo", {
                from: String(from),
                to: String(to),
                total: String(total),
              })
            : t("people.totalCount", { count: 0 })}
        </span>
        <div className="flex gap-2">
          <Button
            variant="outline"
            size="sm"
            onClick={() => onPageChange(page - 1)}
            disabled={page === 0}
          >
            {t("people.previous")}
          </Button>
          <Button
            variant="outline"
            size="sm"
            onClick={() => onPageChange(page + 1)}
            disabled={page + 1 >= totalPages}
          >
            {t("people.next")}
          </Button>
        </div>
      </div>
    </div>
  );
}
