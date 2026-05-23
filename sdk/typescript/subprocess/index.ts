import readline from "node:readline";

/**
 * Thin helper for JSON-RPC-over-stdio subprocess capability packages.
 * Packages can handle host-initiated handshake/invoke requests and can also
 * initiate reverse public `kernel.v1.*` requests (for example
 * `kernel.v1.outbound.execute` and `kernel.v1.outbound.stream`) over the same stdio
 * channel via `kernelClient`.
 */

export type JsonValue = null | boolean | number | string | JsonValue[] | { [key: string]: JsonValue };

export interface JsonRpcRequest {
  [key: string]: unknown;
  jsonrpc?: "2.0";
  id?: string | number | null;
  method?: string;
  params?: Record<string, JsonValue>;
  result?: JsonValue;
  error?: JsonValue;
  kind?: string;
  stream_id?: string;
  data?: JsonValue;
  summary?: JsonValue;
}

export interface CapabilityInvokeParams {
  capability_id: string;
  input?: JsonValue;
}

export interface HandshakeParams {
  protocol_version?: string;
  package_id?: string;
  manifest_version?: string;
  permissions?: JsonValue;
  capabilities?: JsonValue;
}

export type CapabilityHandler = (params: CapabilityInvokeParams) => JsonValue | Promise<JsonValue>;
export type CapabilityHandlerWithContext = (
  params: CapabilityInvokeParams,
  context: { kernelClient: KernelClient },
) => JsonValue | Promise<JsonValue>;
export type HandshakeHandler = (params: HandshakeParams) => JsonValue | Promise<JsonValue>;

export interface SubprocessPackageOptions {
  onHandshake?: HandshakeHandler;
  onInvoke: CapabilityHandler | CapabilityHandlerWithContext;
}

export interface KernelStreamCallbacks {
  onChunk: (chunk: unknown) => void;
  onEnd?: (summary: unknown) => void;
  onError?: (error: unknown) => void;
  onCancelled?: () => void;
  onTimeout?: () => void;
}

export interface KernelStreamHandle {
  readonly streamId: string | undefined;
  cancel(): void;
}

export type KernelWebSocketFrame =
  | { kind: "text"; data: string }
  | { kind: "binary"; data: Uint8Array };

export interface KernelWebSocketHandle {
  readonly connectionId: string;
  readonly subprotocol?: string;
  send(frame: KernelWebSocketFrame): Promise<void>;
  close(code?: number, reason?: string): Promise<void>;
}

export interface KernelWebSocketCallbacks {
  onOpen?: (info: { connectionId: string; subprotocol?: string }) => void;
  onFrame: (frame: KernelWebSocketFrame & { seq: number; direction: "inbound" }) => void;
  onClose?: (info: { code: number; reason: string }) => void;
  onError?: (err: { code: string; message: string }) => void;
}

export interface KernelWebSocketOpenParams {
  capability_id: string;
  destination_host: string;
  path?: string;
  purpose?: string;
  subprotocols?: string[];
  secret_refs?: string[];
  metadata?: Record<string, unknown>;
  static_headers?: Record<string, string>;
  secret_headers?: Record<string, { secret_ref: string; scheme?: string }>;
  max_frame_bytes?: number;
  max_total_bytes_inbound?: number;
  max_total_bytes_outbound?: number;
  max_idle_ms?: number;
  max_duration_ms?: number;
}

export interface KernelClient {
  sendKernelRequest<T = unknown>(method: string, params: unknown): Promise<T>;
  streamKernelRequest(method: string, params: unknown, callbacks: KernelStreamCallbacks): KernelStreamHandle;
  openWebSocket(
    params: KernelWebSocketOpenParams,
    callbacks: KernelWebSocketCallbacks,
  ): Promise<KernelWebSocketHandle>;
}

interface PendingKernelRequest {
  resolve: (value: unknown) => void;
  reject: (error: unknown) => void;
}

interface PendingKernelStream {
  callbacks: KernelStreamCallbacks;
  streamId?: string;
}

interface PendingKernelWebSocketOpen {
  callbacks: KernelWebSocketCallbacks;
  resolve: (handle: KernelWebSocketHandle) => void;
  reject: (error: unknown) => void;
}

interface ActiveKernelWebSocket {
  callbacks: KernelWebSocketCallbacks;
  requestId: string;
  connectionId: string;
  subprotocol?: string;
  closed: boolean;
  closeWaiters: Set<(error: Error) => void>;
}

let nextKernelRequestId = 1;
const pendingKernelRequests = new Map<string, PendingKernelRequest>();
const pendingKernelStreams = new Map<string, PendingKernelStream>();
const streamRequestIdsByStreamId = new Map<string, string>();
const pendingKernelWebSocketOpens = new Map<string, PendingKernelWebSocketOpen>();
const kernelWebSocketsByRequestId = new Map<string, ActiveKernelWebSocket>();
const kernelWebSocketsByConnectionId = new Map<string, ActiveKernelWebSocket>();

function respond(id: JsonRpcRequest["id"], payload: Record<string, JsonValue>) {
  process.stdout.write(JSON.stringify({ jsonrpc: "2.0", id, ...payload }) + "\n");
}

function sendKernelFrame(method: string, params: unknown): string {
  const id = `kreq-${nextKernelRequestId++}`;
  process.stdout.write(JSON.stringify({ jsonrpc: "2.0", id, method, params }) + "\n");
  return id;
}

function rejectKernelError(error: unknown): Error {
  if (error && typeof error === "object" && "message" in error) {
    return new Error(String((error as { message: unknown }).message));
  }
  return new Error(String(error));
}

function asRecord(value: unknown): Record<string, unknown> | undefined {
  return value && typeof value === "object" && !Array.isArray(value) ? value as Record<string, unknown> : undefined;
}

function isWebSocketEventKind(kind: unknown): kind is string {
  return kind === "kernel/v1/outbound.websocket.opened"
    || kind === "kernel/v1/outbound.websocket.frame"
    || kind === "kernel/v1/outbound.websocket.error"
    || kind === "kernel/v1/outbound.websocket.closed"
    || kind === "kernel/v1/outbound.websocket.completed";
}

function getFramePayload(frame: JsonRpcRequest): Record<string, unknown> {
  const record = frame as Record<string, unknown>;
  const payload = asRecord(record.payload) ?? asRecord(record.data) ?? asRecord(record.frame) ?? {};
  return { ...payload, ...record };
}

function getConnectionIdFromFrame(frame: JsonRpcRequest): string | undefined {
  const payload = getFramePayload(frame);
  return typeof payload.connection_id === "string" ? payload.connection_id : undefined;
}

function encodeWebSocketFrame(frame: KernelWebSocketFrame): Record<string, unknown> {
  if (frame.kind === "text") {
    return { kind: "text", data: frame.data };
  }
  return { kind: "binary", bytes: Array.from(frame.data) };
}

function decodeBinaryData(value: unknown): Uint8Array | undefined {
  if (value instanceof Uint8Array) return value;
  if (Array.isArray(value)) {
    const bytes = value.map((item) => typeof item === "number" ? item : Number.NaN);
    if (bytes.every((item) => Number.isInteger(item) && item >= 0 && item <= 255)) {
      return Uint8Array.from(bytes);
    }
    return undefined;
  }
  if (typeof value === "string") {
    return Uint8Array.from(Buffer.from(value, "base64"));
  }
  return undefined;
}

function decodeInboundWebSocketFrame(payload: Record<string, unknown>): KernelWebSocketFrame | undefined {
  const nested = asRecord(payload.frame) ?? asRecord(payload.payload) ?? payload;
  const kind = nested.kind ?? payload.frame_kind;
  if (kind === "text") {
    const data = nested.data ?? nested.text ?? payload.data ?? payload.text;
    return typeof data === "string" ? { kind: "text", data } : undefined;
  }
  if (kind === "binary") {
    const data = nested.bytes ?? nested.data ?? nested.data_b64 ?? nested.payload_b64 ?? payload.bytes ?? payload.data ?? payload.data_b64;
    const decoded = decodeBinaryData(data);
    return decoded ? { kind: "binary", data: decoded } : undefined;
  }
  return undefined;
}

function normalizeSendStatus(status: unknown): string {
  return String(status ?? "ok").toLowerCase().replace(/[^a-z0-9]/g, "");
}

function markWebSocketClosed(session: ActiveKernelWebSocket, error: Error) {
  session.closed = true;
  for (const waiter of session.closeWaiters) waiter(error);
  session.closeWaiters.clear();
}

function removeWebSocketSession(session: ActiveKernelWebSocket) {
  kernelWebSocketsByRequestId.delete(session.requestId);
  kernelWebSocketsByConnectionId.delete(session.connectionId);
}

function createWebSocketHandle(session: ActiveKernelWebSocket): KernelWebSocketHandle {
  return {
    get connectionId() {
      return session.connectionId;
    },
    get subprotocol() {
      return session.subprotocol;
    },
    async send(frame: KernelWebSocketFrame): Promise<void> {
      if (session.closed) {
        throw new Error(`WebSocket connection ${session.connectionId} is closed`);
      }
      let closeReject: ((error: Error) => void) | undefined;
      const closePromise = new Promise<never>((_resolve, reject) => {
        closeReject = reject;
        session.closeWaiters.add(reject);
      });
      try {
        const result = await Promise.race([
          kernelClient.sendKernelRequest<{ status?: unknown }>("kernel.v1.outbound.websocket.send", {
            connection_id: session.connectionId,
            ...encodeWebSocketFrame(frame),
          }),
          closePromise,
        ]);
        const status = normalizeSendStatus(result?.status);
        if (status === "ok") return;
        if (status === "bufferfull") throw new Error(`WebSocket connection ${session.connectionId} send buffer is full`);
        if (status === "connectionclosed") throw new Error(`WebSocket connection ${session.connectionId} is closed`);
        if (status === "connectionnotfound") throw new Error(`WebSocket connection ${session.connectionId} was not found`);
        throw new Error(`WebSocket connection ${session.connectionId} send failed with status ${String(result?.status)}`);
      } finally {
        if (closeReject) session.closeWaiters.delete(closeReject);
      }
    },
    async close(code?: number, reason?: string): Promise<void> {
      if (session.closed) return;
      markWebSocketClosed(session, new Error(`WebSocket connection ${session.connectionId} is closed`));
      await kernelClient.sendKernelRequest("kernel.v1.outbound.websocket.close", {
        connection_id: session.connectionId,
        code,
        reason,
      });
    },
  };
}

function handleWebSocketEvent(session: ActiveKernelWebSocket, frame: JsonRpcRequest): boolean {
  const kind = frame.kind;
  if (!isWebSocketEventKind(kind)) return false;
  const payload = getFramePayload(frame);
  const connectionId = typeof payload.connection_id === "string" ? payload.connection_id : undefined;
  if (connectionId !== session.connectionId) return false;

  switch (kind) {
    case "kernel/v1/outbound.websocket.opened": {
      const subprotocol = typeof payload.subprotocol === "string" ? payload.subprotocol : session.subprotocol;
      if (typeof subprotocol === "string") session.subprotocol = subprotocol;
      session.callbacks.onOpen?.({ connectionId: session.connectionId, subprotocol });
      return true;
    }
    case "kernel/v1/outbound.websocket.frame": {
      const direction = typeof payload.direction === "string" ? payload.direction : "inbound";
      if (direction !== "inbound") return true;
      const decoded = decodeInboundWebSocketFrame(payload);
      const seq = typeof payload.seq === "number" ? payload.seq : Number(payload.sequence ?? 0);
      if (decoded && Number.isFinite(seq)) {
        session.callbacks.onFrame({ ...decoded, seq, direction: "inbound" });
      }
      return true;
    }
    case "kernel/v1/outbound.websocket.error": {
      const code = String(payload.error_code ?? payload.code ?? "websocket_error");
      const message = String(payload.message_redacted ?? payload.message ?? payload.error ?? "WebSocket error");
      session.callbacks.onError?.({ code, message });
      return true;
    }
    case "kernel/v1/outbound.websocket.closed":
    case "kernel/v1/outbound.websocket.completed": {
      const code = typeof payload.code === "number" ? payload.code : Number(payload.code ?? 1000);
      const reason = typeof payload.reason === "string" ? payload.reason : "closed";
      markWebSocketClosed(session, new Error(`WebSocket connection ${session.connectionId} closed: ${code} ${reason}`));
      session.callbacks.onClose?.({ code, reason });
      removeWebSocketSession(session);
      return true;
    }
  }
  return false;
}

function resolveWebSocketOpen(requestId: string, pending: PendingKernelWebSocketOpen, result: unknown) {
  const record = asRecord(result) ?? {};
  const connectionId = record.connection_id;
  if (typeof connectionId !== "string") {
    pending.reject(new Error("kernel.v1.outbound.websocket.open response missing connection_id"));
    return;
  }
  const subprotocol = typeof record.subprotocol_negotiated === "string"
    ? record.subprotocol_negotiated
    : typeof record.subprotocol === "string"
      ? record.subprotocol
      : undefined;
  const session: ActiveKernelWebSocket = {
    callbacks: pending.callbacks,
    requestId,
    connectionId,
    subprotocol,
    closed: false,
    closeWaiters: new Set(),
  };
  kernelWebSocketsByRequestId.set(requestId, session);
  kernelWebSocketsByConnectionId.set(connectionId, session);
  pending.resolve(createWebSocketHandle(session));
}

export const kernelClient: KernelClient = {
  sendKernelRequest<T = unknown>(method: string, params: unknown): Promise<T> {
    const id = sendKernelFrame(method, params);
    return new Promise<T>((resolve, reject) => {
      pendingKernelRequests.set(id, { resolve: resolve as (value: unknown) => void, reject });
    });
  },

  streamKernelRequest(method: string, params: unknown, callbacks: KernelStreamCallbacks): KernelStreamHandle {
    const id = sendKernelFrame(method, params);
    const pending: PendingKernelStream = { callbacks };
    pendingKernelStreams.set(id, pending);
    let cancelled = false;

    return {
      get streamId() {
        return pending.streamId;
      },
      cancel() {
        if (cancelled) return;
        cancelled = true;
        const streamId = pending.streamId;
        if (!streamId) return;
        sendKernelFrame("kernel.v1.capability.cancel", { stream_id: streamId, invocation_id: streamId, session_id: `subprocess_reverse_${streamId}` });
      },
    };
  },

  openWebSocket(params: KernelWebSocketOpenParams, callbacks: KernelWebSocketCallbacks): Promise<KernelWebSocketHandle> {
    const id = sendKernelFrame("kernel.v1.outbound.websocket.open", params);
    return new Promise<KernelWebSocketHandle>((resolve, reject) => {
      pendingKernelWebSocketOpens.set(id, { callbacks, resolve, reject });
    });
  },
};

function handleKernelInbound(frame: JsonRpcRequest): boolean {
  if (typeof frame.id !== "string" || !frame.id.startsWith("kreq-")) {
    const connectionId = getConnectionIdFromFrame(frame);
    const session = connectionId ? kernelWebSocketsByConnectionId.get(connectionId) : undefined;
    return session ? handleWebSocketEvent(session, frame) : false;
  }
  const requestId = frame.id;

  const pendingWebSocketOpen = pendingKernelWebSocketOpens.get(requestId);
  if (pendingWebSocketOpen) {
    if (frame.result && typeof frame.result === "object") {
      pendingKernelWebSocketOpens.delete(requestId);
      resolveWebSocketOpen(requestId, pendingWebSocketOpen, frame.result);
      return true;
    }
    if (frame.error) {
      pendingKernelWebSocketOpens.delete(requestId);
      pendingWebSocketOpen.reject(rejectKernelError(frame.error));
      return true;
    }
  }

  const websocket = kernelWebSocketsByRequestId.get(requestId);
  if (websocket && handleWebSocketEvent(websocket, frame)) return true;

  const pendingStream = pendingKernelStreams.get(requestId);
  if (pendingStream) {
    if (frame.result && typeof frame.result === "object") {
      const streamId = (frame.result as { stream_id?: unknown }).stream_id;
      if (typeof streamId === "string") {
        pendingStream.streamId = streamId;
        streamRequestIdsByStreamId.set(streamId, requestId);
      }
      return true;
    }
    if (frame.error) {
      pendingKernelStreams.delete(requestId);
      pendingStream.callbacks.onError?.(frame.error);
      return true;
    }

    switch (frame.kind) {
      case "kernel/v1/stream.chunk":
      case "stream.chunk":
        pendingStream.callbacks.onChunk(frame.data);
        return true;
      case "kernel/v1/stream.ended":
      case "stream.ended":
        pendingKernelStreams.delete(requestId);
        if (pendingStream.streamId) streamRequestIdsByStreamId.delete(pendingStream.streamId);
        pendingStream.callbacks.onEnd?.(frame.summary);
        return true;
      case "kernel/v1/stream.error":
      case "stream.error":
        pendingKernelStreams.delete(requestId);
        if (pendingStream.streamId) streamRequestIdsByStreamId.delete(pendingStream.streamId);
        pendingStream.callbacks.onError?.(frame.error);
        return true;
      case "kernel/v1/stream.cancelled":
      case "stream.cancelled":
        pendingKernelStreams.delete(requestId);
        if (pendingStream.streamId) streamRequestIdsByStreamId.delete(pendingStream.streamId);
        pendingStream.callbacks.onCancelled?.();
        return true;
      case "kernel/v1/stream.timeout":
      case "stream.timeout":
        pendingKernelStreams.delete(requestId);
        if (pendingStream.streamId) streamRequestIdsByStreamId.delete(pendingStream.streamId);
        pendingStream.callbacks.onTimeout?.();
        return true;
      default:
        return false;
    }
  }

  const pending = pendingKernelRequests.get(requestId);
  if (!pending) return false;
  pendingKernelRequests.delete(requestId);
  if (frame.error) pending.reject(rejectKernelError(frame.error));
  else pending.resolve(frame.result);
  return true;
}

export const __handleKernelInboundForTest = handleKernelInbound;

export function serveSubprocessPackage(options: SubprocessPackageOptions) {
  const rl = readline.createInterface({ input: process.stdin, crlfDelay: Infinity });
  rl.on("line", async (line) => {
    let request: JsonRpcRequest;
    try {
      request = JSON.parse(line);
    } catch (error) {
      respond(null, { error: { code: "invalid_json", message: String(error) } as JsonValue });
      return;
    }

    if (handleKernelInbound(request)) return;

    try {
      if (request.method === "package.handshake") {
        const result = options.onHandshake
          ? await options.onHandshake((request.params ?? {}) as HandshakeParams)
          : { ready: true, package_protocol_version: "0.1.0" };
        respond(request.id, { result: result as JsonValue });
      } else if (request.method === "capability.invoke") {
        const output = await options.onInvoke((request.params ?? {}) as unknown as CapabilityInvokeParams, { kernelClient });
        respond(request.id, { result: { output } as JsonValue });
      } else {
        respond(request.id, { error: { code: "unknown_method", message: request.method ?? "<missing>" } as JsonValue });
      }
    } catch (error) {
      respond(request.id, { error: { code: "package_error", message: String(error) } as JsonValue });
    }
  });
}
