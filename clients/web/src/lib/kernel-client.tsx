/**
 * React-friendly access to the kernel protocol client.
 * Wraps the existing YggProtocolClient with a context + hooks.
 */

import {
  createContext,
  useContext,
  useEffect,
  useMemo,
  useRef,
  useState,
  type ReactNode,
} from "react";
import { YggProtocolClient, type KernelEvent } from "@/protocol/client";
import { resolveHostBaseUrl } from "@/client-core/host-endpoint";

interface KernelContextValue {
  client: YggProtocolClient;
}

const KernelContext = createContext<KernelContextValue | null>(null);

export function KernelProvider({
  children,
  baseUrl,
  accessToken,
}: {
  children: ReactNode;
  baseUrl?: string;
  accessToken?: string | null;
}) {
  const client = useMemo(() => {
    const url = resolveHostBaseUrl(baseUrl);
    return new YggProtocolClient(url, accessToken);
  }, [baseUrl, accessToken]);

  return <KernelContext.Provider value={{ client }}>{children}</KernelContext.Provider>;
}

export function useKernel(): YggProtocolClient {
  const ctx = useContext(KernelContext);
  if (!ctx) throw new Error("useKernel must be used inside <KernelProvider>");
  return ctx.client;
}

export interface AsyncResource<T> {
  data?: T;
  error?: Error;
  loading: boolean;
  refresh: () => void;
}

export function useAsync<T>(fn: () => Promise<T>, deps: unknown[] = []): AsyncResource<T> {
  const [data, setData] = useState<T | undefined>(undefined);
  const [error, setError] = useState<Error | undefined>(undefined);
  const [loading, setLoading] = useState(true);
  const [tick, setTick] = useState(0);
  // Monotonic request id avoids the stale-shared-ref race where a refresh
  // resets `cancelled.current = false` mid-flight and lets an older promise
  // overwrite newer data.
  const reqIdRef = useRef(0);

  useEffect(() => {
    const myReqId = ++reqIdRef.current;
    setLoading(true);
    setError(undefined);
    fn()
      .then((value) => {
        if (reqIdRef.current === myReqId) {
          setData(value);
          setLoading(false);
        }
      })
      .catch((err: unknown) => {
        if (reqIdRef.current === myReqId) {
          setError(err instanceof Error ? err : new Error(String(err)));
          setLoading(false);
        }
      });
    return () => {
      // Bumping the id invalidates any in-flight promise from this effect.
      reqIdRef.current++;
    };
    // eslint-disable-next-line react-hooks/exhaustive-deps
  }, [...deps, tick]);

  return { data, error, loading, refresh: () => setTick((n) => n + 1) };
}

export function useEventTail(sessionId: string | undefined, limit = 24): KernelEvent[] {
  const client = useKernel();
  const [events, setEvents] = useState<KernelEvent[]>([]);

  useEffect(() => {
    if (!sessionId) return;
    let cancelled = false;
    client
      .listEvents(sessionId)
      .then((records) => {
        if (!cancelled) setEvents(records.slice(-limit));
      })
      .catch(() => {
        // ignored — tail is best-effort
      });
    const close = client.subscribeEvents(sessionId, (event) => {
      setEvents((current) => [...current, event].slice(-limit));
    });
    return () => {
      cancelled = true;
      close();
    };
  }, [client, sessionId, limit]);

  return events;
}
