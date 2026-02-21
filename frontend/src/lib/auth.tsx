"use client";

import { createContext, useContext, type ReactNode } from "react";
import { env } from "@/lib/env";

interface AuthContextValue {
  orgId: string;
  userId: string | null;
  role: "admin" | "manager" | "analyst" | "viewer";
}

const AuthContext = createContext<AuthContextValue>({
  orgId: env.NEXT_PUBLIC_ORG_ID,
  userId: null,
  role: "admin",
});

export function AuthProvider({ children }: { children: ReactNode }) {
  // Stub: replace with real auth (JWT/cookie) in Phase 2+
  const value: AuthContextValue = {
    orgId: env.NEXT_PUBLIC_ORG_ID,
    userId: null,
    role: "admin",
  };

  return <AuthContext.Provider value={value}>{children}</AuthContext.Provider>;
}

export function useAuth() {
  return useContext(AuthContext);
}
