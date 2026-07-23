import { createContext, useContext, useEffect, useMemo, useRef, useState, useCallback, type ReactNode } from "react";
import {
  ProtocolHttpError,
  YggProtocolClient,
} from "@/protocol/client";
import {
  clearBrowserAccessToken,
  PendingCredentialLease,
  readBrowserAccessToken,
  storeBrowserAccessToken,
} from "@/client-core/credentials";
import { resolveHostBaseUrl } from "@/client-core/host-endpoint";
import { useT } from "@/lib/locale";

export type AuthStatus = "checking" | "optional" | "required" | "authenticated" | "invalid" | "unavailable";

interface AuthContextValue {
  status: AuthStatus;
  token: string | null;
  error: string | null;
  login: (token: string) => Promise<void>;
  logout: () => void;
  retry: () => Promise<void>;
}

const AuthContext = createContext<AuthContextValue | null>(null);

async function probeServer(client: YggProtocolClient): Promise<{ authError: boolean }> {
  try {
    await client.diagnostics();
    return { authError: false };
  } catch (err) {
    if (err instanceof ProtocolHttpError && err.isAuthError) {
      return { authError: true };
    }
    throw err;
  }
}

type ProbeResult = "ok" | "auth-error";

export function AuthProvider({ children }: { children: ReactNode }) {
  const t = useT();
  const [status, setStatus] = useState<AuthStatus>("checking");
  const [token, setToken] = useState<string | null>(null);
  const [error, setError] = useState<string | null>(null);
  const probeIdRef = useRef(0);
  const pendingCredentialRef = useRef<PendingCredentialLease | null>(null);
  if (!pendingCredentialRef.current) pendingCredentialRef.current = new PendingCredentialLease();

  const validateToken = useCallback(async (candidateToken: string): Promise<ProbeResult> => {
    const baseUrl = resolveHostBaseUrl();
    const client = new YggProtocolClient(baseUrl, candidateToken);
    const { authError } = await probeServer(client);
    return authError ? "auth-error" : "ok";
  }, []);

  const authenticateCandidate = useCallback(
    async (candidateToken: string, probeId: number) => {
      pendingCredentialRef.current?.retain(candidateToken);
      try {
        const result = await validateToken(candidateToken);
        if (probeIdRef.current !== probeId) return;
        if (result === "ok") {
          storeBrowserAccessToken(candidateToken);
          pendingCredentialRef.current?.clear();
          setToken(candidateToken);
          setStatus("authenticated");
        } else {
          clearBrowserAccessToken();
          pendingCredentialRef.current?.clear();
          setToken(null);
          setError(t("authInvalidToken"));
          setStatus("invalid");
        }
      } catch (err) {
        if (probeIdRef.current !== probeId) return;
        const msg = err instanceof Error ? err.message : String(err);
        setToken(candidateToken);
        setError(t("authConnectionFailed", msg));
        setStatus("unavailable");
      }
    },
    [t, validateToken],
  );

  const runStartupProbe = useCallback(async () => {
    const probeId = ++probeIdRef.current;
    setStatus("checking");
    setError(null);
    // Consume and scrub one-time Desktop/deep-link credentials immediately,
    // even when this Host later reports that authentication is optional.
    const candidateToken = pendingCredentialRef.current?.resolve() ?? readBrowserAccessToken() ?? null;
    const baseUrl = resolveHostBaseUrl();
    const client = new YggProtocolClient(baseUrl, null);
    try {
      const { authError } = await probeServer(client);
      if (probeIdRef.current !== probeId) return;
      if (!authError) {
        pendingCredentialRef.current?.clear();
        setToken(null);
        setStatus("optional");
      } else {
        if (candidateToken) {
          await authenticateCandidate(candidateToken, probeId);
        } else {
          setToken(null);
          setStatus("required");
        }
      }
    } catch (err) {
      if (probeIdRef.current !== probeId) return;
      const msg = err instanceof Error ? err.message : String(err);
      setError(t("authConnectionFailed", msg));
      setToken(candidateToken);
      setStatus("unavailable");
    }
  }, [authenticateCandidate, t]);

  const runLoginProbe = useCallback(
    async (candidateToken: string) => {
      const probeId = ++probeIdRef.current;
      setStatus("checking");
      setError(null);
      await authenticateCandidate(candidateToken, probeId);
    },
    [authenticateCandidate],
  );

  useEffect(() => {
    runStartupProbe();
    // eslint-disable-next-line react-hooks/exhaustive-deps
  }, []);

  useEffect(() => {
    if (typeof window === "undefined") return;
    const retryWhenOnline = () => void runStartupProbe();
    window.addEventListener("online", retryWhenOnline);
    return () => window.removeEventListener("online", retryWhenOnline);
  }, [runStartupProbe]);

  const login = useCallback(
    async (newToken: string) => {
      const trimmed = newToken.trim();
      if (!trimmed) return;
      await runLoginProbe(trimmed);
    },
    [runLoginProbe],
  );

  const logout = useCallback(() => {
    clearBrowserAccessToken();
    pendingCredentialRef.current?.clear();
    setToken(null);
    setError(null);
    runStartupProbe();
  }, [runStartupProbe]);

  const value = useMemo<AuthContextValue>(
    () => ({ status, token, error, login, logout, retry: runStartupProbe }),
    [status, token, error, login, logout, runStartupProbe],
  );

  return <AuthContext.Provider value={value}>{children}</AuthContext.Provider>;
}

export function useAuth(): AuthContextValue {
  const ctx = useContext(AuthContext);
  if (!ctx) throw new Error("useAuth must be used inside <AuthProvider>");
  return ctx;
}
