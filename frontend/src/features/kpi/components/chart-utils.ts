import type { Locale } from "@/i18n";
import { formatShortDate } from "@/i18n";

/** Format "2025-11-24" → "Nov 24" (en) / "24 нояб." (ru) */
export function formatWeekLabel(periodStart: string, locale: Locale = "en"): string {
  return formatShortDate(periodStart, locale);
}
