import { createContext, useContext, useEffect, useMemo, useRef, useState, useCallback, type ReactNode } from "react";
import {
  clearBrowserAccessToken,
  ProtocolHttpError,
  readBrowserAccessToken,
  resolveBrowserAccessToken,
  storeBrowserAccessToken,
  YggProtocolClient,
} from "@/protocol/client";

export type AuthStatus = "checking" | "optional" | "required" | "authenticated" | "invalid";

interface AuthContextValue {
  status: AuthStatus;
  token: string | null;
  error: string | null;
  login: (token: string) => Promise<void>;
  logout: () => void;
}

const AuthContext = createContext<AuthContextValue | null>(null);

function getBaseUrl() {
  return typeof location !== "undefined" && location.origin !== "null" ? location.origin : "http://127.0.0.1:8787";
}

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
  const [status, setStatus] = useState<AuthStatus>("checking");
  const [token, setToken] = useState<string | null>(null);
  const [error, setError] = useState<string | null>(null);
  const probeIdRef = useRef(0);

  const validateToken = useCallback(async (candidateToken: string): Promise<ProbeResult> => {
    const baseUrl = getBaseUrl();
    const client = new YggProtocolClient(baseUrl, candidateToken);
    const { authError } = await probeServer(client);
    return authError ? "auth-error" : "ok";
  }, []);

  const authenticateCandidate = useCallback(
    async (candidateToken: string, probeId: number) => {
      try {
        const result = await validateToken(candidateToken);
        if (probeIdRef.current !== probeId) return;
        if (result === "ok") {
          storeBrowserAccessToken(candidateToken);
          setToken(candidateToken);
          setStatus("authenticated");
        } else {
          clearBrowserAccessToken();
          setToken(null);
          setError("Invalid access token. Please check your token and try again.");
          setStatus("invalid");
        }
      } catch (err) {
        if (probeIdRef.current !== probeId) return;
        const msg = err instanceof Error ? err.message : String(err);
        setToken(null);
        setError(`Connection failed: ${msg}`);
        setStatus("invalid");
      }
    },
    [validateToken],
  );

  const runStartupProbe = useCallback(async () => {
    const probeId = ++probeIdRef.current;
    setStatus("checking");
    setError(null);
    const baseUrl = getBaseUrl();
    const client = new YggProtocolClient(baseUrl, null);
    try {
      const { authError } = await probeServer(client);
      if (probeIdRef.current !== probeId) return;
      if (!authError) {
        setToken(null);
        setStatus("optional");
      } else {
        const bootstrapToken = resolveBrowserAccessToken() ?? readBrowserAccessToken() ?? null;
        if (bootstrapToken) {
          await authenticateCandidate(bootstrapToken, probeId);
        } else {
          setToken(null);
          setStatus("required");
        }
      }
    } catch (err) {
      if (probeIdRef.current !== probeId) return;
      const msg = err instanceof Error ? err.message : String(err);
      setError(`Connection failed: ${msg}`);
      setToken(null);
      setStatus("required");
    }
  }, [authenticateCandidate]);

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
    setToken(null);
    setError(null);
    runStartupProbe();
  }, [runStartupProbe]);

  const value = useMemo<AuthContextValue>(
    () => ({ status, token, error, login, logout }),
    [status, token, error, login, logout],
  );

  return <AuthContext.Provider value={value}>{children}</AuthContext.Provider>;
}

export function useAuth(): AuthContextValue {
  const ctx = useContext(AuthContext);
  if (!ctx) throw new Error("useAuth must be used inside <AuthProvider>");
  return ctx;
}
