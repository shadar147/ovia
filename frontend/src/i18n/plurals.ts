import type { Locale } from "./types";

export type PluralForm = "one" | "few" | "many";

export function getPluralForm(count: number, locale: Locale): PluralForm {
  if (locale === "ru") {
    return getRussianPluralForm(count);
  }
  return count === 1 ? "one" : "many";
}

/**
 * Russian plural rules:
 *  - "one"  → 1, 21, 31, … (but not 11, 111, …)
 *  - "few"  → 2-4, 22-24, 32-34, … (but not 12-14, 112-114, …)
 *  - "many" → everything else (0, 5-20, 25-30, 11-14, …)
 */
function getRussianPluralForm(count: number): PluralForm {
  const abs = Math.abs(count);
  const mod10 = abs % 10;
  const mod100 = abs % 100;

  if (mod10 === 1 && mod100 !== 11) return "one";
  if (mod10 >= 2 && mod10 <= 4 && (mod100 < 12 || mod100 > 14)) return "few";
  return "many";
}
