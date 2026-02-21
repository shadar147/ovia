"use client";

import {
  useReactTable,
  getCoreRowModel,
  getSortedRowModel,
  getPaginationRowModel,
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
import { ArrowUpDown } from "lucide-react";
import { StatusBadge, ConfidenceBadge } from "./status-badge";
import type { PersonIdentityLink } from "@/lib/api/types";
import { useTranslation, formatShortDate } from "@/i18n";
import type { MessageKey } from "@/i18n";

interface MappingTableProps {
  data: PersonIdentityLink[];
  onRowClick: (link: PersonIdentityLink) => void;
}

function SortableHeader({ column, labelKey }: { column: { toggleSorting: (desc: boolean) => void; getIsSorted: () => false | "asc" | "desc" }; labelKey: MessageKey }) {
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

export function MappingTable({ data, onRowClick }: MappingTableProps) {
  const [sorting, setSorting] = useState<SortingState>([]);
  const { t, locale } = useTranslation();

  const columns: ColumnDef<PersonIdentityLink>[] = useMemo(
    () => [
      {
        accessorKey: "person",
        header: () => t("identity.colPerson"),
        cell: ({ row }) => {
          const person = row.original.person;
          return (
            <div>
              <p className="font-medium">{person?.display_name ?? "—"}</p>
              <p className="text-xs text-muted-foreground">{person?.primary_email ?? ""}</p>
            </div>
          );
        },
        enableSorting: false,
      },
      {
        accessorKey: "identity",
        header: () => t("identity.colIdentity"),
        cell: ({ row }) => {
          const identity = row.original.identity;
          return (
            <div>
              <p className="font-medium">
                {identity?.display_name ?? identity?.username ?? "—"}
              </p>
              <p className="text-xs text-muted-foreground">
                {identity?.source} {identity?.email ? `· ${identity.email}` : ""}
              </p>
            </div>
          );
        },
        enableSorting: false,
      },
      {
        accessorKey: "status",
        header: ({ column }) => <SortableHeader column={column} labelKey="identity.colStatus" />,
        cell: ({ row }) => <StatusBadge status={row.original.status} />,
      },
      {
        accessorKey: "confidence",
        header: ({ column }) => <SortableHeader column={column} labelKey="identity.colConfidence" />,
        cell: ({ row }) => <ConfidenceBadge confidence={row.original.confidence} />,
      },
      {
        accessorKey: "updated_at",
        header: ({ column }) => <SortableHeader column={column} labelKey="identity.colUpdated" />,
        cell: ({ row }) => (
          <span className="text-sm text-muted-foreground">
            {formatShortDate(row.original.updated_at.split("T")[0] ?? row.original.updated_at, locale)}
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
    getPaginationRowModel: getPaginationRowModel(),
    onSortingChange: setSorting,
    state: { sorting },
    initialState: { pagination: { pageSize: 25 } },
  });

  return (
    <div>
      <div className="rounded-md border">
        <Table>
          <TableHeader>
            {table.getHeaderGroups().map((hg) => (
              <TableRow key={hg.id}>
                {hg.headers.map((h) => (
                  <TableHead key={h.id} className="h-12">
                    {h.isPlaceholder ? null : flexRender(h.column.columnDef.header, h.getContext())}
                  </TableHead>
                ))}
              </TableRow>
            ))}
          </TableHeader>
          <TableBody>
            {table.getRowModel().rows.length === 0 ? (
              <TableRow>
                <TableCell colSpan={columns.length} className="h-24 text-center">
                  {t("identity.noMappingsFound")}
                </TableCell>
              </TableRow>
            ) : (
              table.getRowModel().rows.map((row) => (
                <TableRow
                  key={row.id}
                  className="cursor-pointer hover:bg-muted/50"
                  onClick={() => onRowClick(row.original)}
                >
                  {row.getVisibleCells().map((cell) => (
                    <TableCell key={cell.id} className="h-[52px]">
                      {flexRender(cell.column.columnDef.cell, cell.getContext())}
                    </TableCell>
                  ))}
                </TableRow>
              ))
            )}
          </TableBody>
        </Table>
      </div>

      {/* Pagination */}
      <div className="flex items-center justify-between py-4">
        <span className="text-sm text-muted-foreground">
          {t("identity.totalCount", { count: table.getFilteredRowModel().rows.length })}
        </span>
        <div className="flex gap-2">
          <Button
            variant="outline"
            size="sm"
            onClick={() => table.previousPage()}
            disabled={!table.getCanPreviousPage()}
          >
            {t("identity.previous")}
          </Button>
          <Button
            variant="outline"
            size="sm"
            onClick={() => table.nextPage()}
            disabled={!table.getCanNextPage()}
          >
            {t("identity.next")}
          </Button>
        </div>
      </div>
    </div>
  );
}
