import { create } from "zustand";
import { persist } from "zustand/middleware";
import type { Locale } from "./types";

interface LocaleState {
  locale: Locale;
  setLocale: (locale: Locale) => void;
  toggleLocale: () => void;
}

export const useLocaleStore = create<LocaleState>()(
  persist(
    (set, get) => ({
      locale: "en",
      setLocale: (locale) => set({ locale }),
      toggleLocale: () => set({ locale: get().locale === "en" ? "ru" : "en" }),
    }),
    { name: "ovia-locale" },
  ),
);
