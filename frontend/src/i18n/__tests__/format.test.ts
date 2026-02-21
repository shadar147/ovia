import { describe, it, expect } from "vitest";
import { formatShortDate, formatDateRange } from "../format";

describe("formatShortDate", () => {
  it("formats date in English", () => {
    const result = formatShortDate("2025-11-24", "en");
    expect(result).toMatch(/Nov\s+24/);
  });

  it("formats date in Russian", () => {
    const result = formatShortDate("2025-11-24", "ru");
    // Russian uses "нояб." or "24 нояб." depending on locale impl
    expect(result).toMatch(/24/);
    expect(result).toMatch(/нояб/i);
  });
});

describe("formatDateRange", () => {
  it("formats date range in English", () => {
    const result = formatDateRange("2025-11-24", "2025-12-01", "en");
    expect(result).toMatch(/Nov\s+24/);
    expect(result).toContain("-");
    expect(result).toMatch(/Dec\s+1/);
  });

  it("formats date range in Russian", () => {
    const result = formatDateRange("2025-11-24", "2025-12-01", "ru");
    expect(result).toMatch(/нояб/i);
    expect(result).toContain("-");
    expect(result).toMatch(/дек/i);
  });
});
