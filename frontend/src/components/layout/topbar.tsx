"use client";

import { useAuth } from "@/lib/auth";
import { useLocaleStore, useTranslation } from "@/i18n";
import { Globe } from "lucide-react";
import { Button } from "@/components/ui/button";

export function Topbar() {
  const { orgId, role } = useAuth();
  const { t, locale } = useTranslation();
  const toggleLocale = useLocaleStore((s) => s.toggleLocale);

  return (
    <header className="flex h-16 items-center justify-between border-b px-6">
      <div />
      <div className="flex items-center gap-4 text-sm text-muted-foreground">
        <span className="hidden sm:inline">{t("topbar.org", { org: orgId.slice(0, 8) + "..." })}</span>
        <span className="rounded-full bg-muted px-2.5 py-0.5 text-xs font-medium">{role}</span>
        <Button
          variant="ghost"
          size="sm"
          className="h-8 gap-1.5 px-2"
          onClick={toggleLocale}
        >
          <Globe className="h-4 w-4" />
          <span className="text-xs font-medium">{locale.toUpperCase()}</span>
        </Button>
      </div>
    </header>
  );
}
