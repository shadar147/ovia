"use client";

import { useTranslation } from "@/i18n";

export default function ReportsPage() {
  const { t } = useTranslation();

  return (
    <div>
      <h1 className="text-2xl font-semibold">{t("reports.title")}</h1>
      <p className="mt-2 text-muted-foreground">{t("reports.description")}</p>
    </div>
  );
}
