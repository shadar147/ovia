import type { Locale } from "./types";

const localeMap: Record<Locale, string> = {
  en: "en-US",
  ru: "ru-RU",
};

/** "Nov 24" (en) / "24 нояб." (ru) */
export function formatShortDate(dateStr: string, locale: Locale): string {
  const d = new Date(dateStr + "T00:00:00");
  return d.toLocaleDateString(localeMap[locale], { month: "short", day: "numeric" });
}

/** "Nov 24" - "Dec 1" (en) / "24 нояб. - 1 дек." (ru) */
export function formatDateRange(start: string, end: string, locale: Locale): string {
  return `${formatShortDate(start, locale)} - ${formatShortDate(end, locale)}`;
}

/** Full locale-aware date+time string */
export function formatDateTime(dateStr: string, locale: Locale): string {
  const d = new Date(dateStr);
  return d.toLocaleString(localeMap[locale]);
}
