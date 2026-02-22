import type { Locale } from "@/i18n";
import { formatShortDate } from "@/i18n";
import type { KpiSnapshot } from "@/lib/api/types";

/** Format "2025-11-24" → "Nov 24" (en) / "24 нояб." (ru) */
export function formatWeekLabel(periodStart: string, locale: Locale = "en"): string {
  return formatShortDate(periodStart, locale);
}

/**
 * Deduplicate history snapshots by period_start.
 * When multiple snapshots share the same period_start (e.g. weekly + monthly),
 * keep only the one with the latest computed_at.
 * Returns a new array sorted by period_start ascending.
 */
export function deduplicateHistory(snapshots: KpiSnapshot[]): KpiSnapshot[] {
  const map = new Map<string, KpiSnapshot>();
  for (const s of snapshots) {
    const existing = map.get(s.period_start);
    if (!existing || s.computed_at > existing.computed_at) {
      map.set(s.period_start, s);
    }
  }
  return Array.from(map.values()).sort(
    (a, b) => a.period_start.localeCompare(b.period_start),
  );
}
