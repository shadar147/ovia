import { z } from "zod/v4";

const envSchema = z.object({
  NEXT_PUBLIC_API_URL: z.string().default("http://localhost:8080"),
  NEXT_PUBLIC_ORG_ID: z.string().default("00000000-0000-0000-0000-000000000001"),
});

export const env = envSchema.parse({
  NEXT_PUBLIC_API_URL: process.env.NEXT_PUBLIC_API_URL,
  NEXT_PUBLIC_ORG_ID: process.env.NEXT_PUBLIC_ORG_ID,
});
