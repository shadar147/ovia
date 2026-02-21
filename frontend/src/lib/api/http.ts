import { env } from "@/lib/env";

export class ApiError extends Error {
  constructor(
    public status: number,
    public statusText: string,
    public body?: unknown,
  ) {
    super(`${status} ${statusText}`);
    this.name = "ApiError";
  }
}

interface RequestOptions extends Omit<RequestInit, "body"> {
  body?: unknown;
  timeout?: number;
}

export async function api<T>(path: string, options: RequestOptions = {}): Promise<T> {
  const { body, timeout = 10000, headers: extraHeaders, ...rest } = options;

  const controller = new AbortController();
  const timer = setTimeout(() => controller.abort(), timeout);

  const headers: Record<string, string> = {
    "x-org-id": env.NEXT_PUBLIC_ORG_ID,
    ...(extraHeaders as Record<string, string>),
  };

  if (body !== undefined) {
    headers["content-type"] = "application/json";
  }

  try {
    const res = await fetch(`${env.NEXT_PUBLIC_API_URL}${path}`, {
      ...rest,
      headers,
      body: body !== undefined ? JSON.stringify(body) : undefined,
      signal: controller.signal,
    });

    if (!res.ok) {
      const errorBody = await res.json().catch(() => null);
      throw new ApiError(res.status, res.statusText, errorBody);
    }

    if (res.status === 204) return undefined as T;

    return (await res.json()) as T;
  } finally {
    clearTimeout(timer);
  }
}
