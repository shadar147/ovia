"use client";

import { useAuth } from "@/lib/auth";

export function Topbar() {
  const { orgId, role } = useAuth();

  return (
    <header className="flex h-16 items-center justify-between border-b px-6">
      <div />
      <div className="flex items-center gap-4 text-sm text-muted-foreground">
        <span className="hidden sm:inline">org: {orgId.slice(0, 8)}...</span>
        <span className="rounded-full bg-muted px-2.5 py-0.5 text-xs font-medium">{role}</span>
      </div>
    </header>
  );
}
