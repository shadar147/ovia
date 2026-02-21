import { useLocaleStore } from "./store";
import { messages as en } from "./messages/en";
import { messages as ru } from "./messages/ru";
import { getPluralForm } from "./plurals";
import type { Locale, MessageKey } from "./types";

const catalogs: Record<Locale, Record<string, string>> = { en, ru };

type TranslateParams = Record<string, string | number>;

function resolve(key: string, locale: Locale): string {
  return catalogs[locale]?.[key] ?? catalogs.en[key] ?? key;
}

function interpolate(template: string, params: TranslateParams): string {
  return template.replace(/\{(\w+)\}/g, (_, name: string) => {
    const val = params[name];
    return val !== undefined ? String(val) : `{${name}}`;
  });
}

function translate(key: MessageKey, locale: Locale, params?: TranslateParams): string {
  if (params && typeof params.count === "number") {
    const form = getPluralForm(params.count, locale);
    const pluralKey = `${key}_${form}`;
    const pluralTemplate = catalogs[locale]?.[pluralKey] ?? catalogs.en[pluralKey];
    if (pluralTemplate) {
      return interpolate(pluralTemplate, params);
    }
  }
  const template = resolve(key, locale);
  return params ? interpolate(template, params) : template;
}

export function useTranslation() {
  const locale = useLocaleStore((s) => s.locale);

  function t(key: MessageKey, params?: TranslateParams): string {
    return translate(key, locale, params);
  }

  return { t, locale } as const;
}
