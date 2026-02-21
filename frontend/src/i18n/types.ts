import type { messages as en } from "./messages/en";

export type Locale = "en" | "ru";

export type MessageKey = keyof typeof en;

export type Messages = Record<MessageKey, string>;
