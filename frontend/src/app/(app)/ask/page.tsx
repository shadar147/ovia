"use client";

import { useTranslation } from "@/i18n";

export default function AskPage() {
  const { t } = useTranslation();

  return (
    <div>
      <h1 className="text-2xl font-semibold">{t("ask.title")}</h1>
      <p className="mt-2 text-muted-foreground">{t("ask.description")}</p>
    </div>
  );
}
