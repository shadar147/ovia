"use client";

import { useEffect } from "react";
import { useLocaleStore } from "./store";

export function HtmlLangSync() {
  const locale = useLocaleStore((s) => s.locale);

  useEffect(() => {
    document.documentElement.lang = locale;
  }, [locale]);

  return null;
}
