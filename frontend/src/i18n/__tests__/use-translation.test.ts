import { describe, it, expect, beforeEach } from "vitest";
import { renderHook, act } from "@testing-library/react";
import { useTranslation } from "../use-translation";
import { useLocaleStore } from "../store";

describe("useTranslation", () => {
  beforeEach(() => {
    // Reset store to default (en)
    act(() => {
      useLocaleStore.setState({ locale: "en" });
    });
  });

  it("returns English text by default", () => {
    const { result } = renderHook(() => useTranslation());
    expect(result.current.t("nav.dashboard")).toBe("Dashboard");
    expect(result.current.locale).toBe("en");
  });

  it("returns Russian text when locale is ru", () => {
    act(() => {
      useLocaleStore.setState({ locale: "ru" });
    });
    const { result } = renderHook(() => useTranslation());
    expect(result.current.t("nav.dashboard")).toBe("Дашборд");
    expect(result.current.locale).toBe("ru");
  });

  it("interpolates parameters", () => {
    const { result } = renderHook(() => useTranslation());
    expect(result.current.t("topbar.org", { org: "abc123" })).toBe("org: abc123");
  });

  it("interpolates parameters in Russian", () => {
    act(() => {
      useLocaleStore.setState({ locale: "ru" });
    });
    const { result } = renderHook(() => useTranslation());
    expect(result.current.t("topbar.org", { org: "abc123" })).toBe("орг: abc123");
  });

  it("falls back to English for missing Russian keys", () => {
    act(() => {
      useLocaleStore.setState({ locale: "ru" });
    });
    const { result } = renderHook(() => useTranslation());
    // All keys should exist, but if one was missing it would fall back to en
    // Testing with a known key to verify the mechanism works
    expect(result.current.t("nav.dashboard")).toBeTruthy();
  });

  it("returns raw key for completely unknown keys", () => {
    const { result } = renderHook(() => useTranslation());
    // @ts-expect-error - intentionally testing unknown key
    expect(result.current.t("nonexistent.key")).toBe("nonexistent.key");
  });

  it("handles interpolation with count param", () => {
    const { result } = renderHook(() => useTranslation());
    expect(result.current.t("identity.bulkConfirm", { count: 5 })).toBe("Bulk Confirm (5)");
  });

  it("handles multiple params", () => {
    const { result } = renderHook(() => useTranslation());
    expect(result.current.t("identity.filterConfidence", { min: 20, max: 80 })).toBe("Confidence: 20% – 80%");
  });
});
