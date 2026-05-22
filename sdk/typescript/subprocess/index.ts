import readline from "node:readline";

/**
 * Thin helper for JSON-RPC-over-stdio subprocess capability packages.
 * Packages can handle host-initiated handshake/invoke requests and can also
 * initiate reverse public `kernel.*` requests (for example
 * `kernel.outbound.execute` and `kernel.outbound.stream`) over the same stdio
 * channel via `kernelClient`.
 */

export type JsonValue = null | boolean | number | string | JsonValue[] | { [key: string]: JsonValue };

export interface JsonRpcRequest {
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

export interface KernelClient {
  sendKernelRequest<T = unknown>(method: string, params: unknown): Promise<T>;
  streamKernelRequest(method: string, params: unknown, callbacks: KernelStreamCallbacks): KernelStreamHandle;
}

interface PendingKernelRequest {
  resolve: (value: unknown) => void;
  reject: (error: unknown) => void;
}

interface PendingKernelStream {
  callbacks: KernelStreamCallbacks;
  streamId?: string;
}

let nextKernelRequestId = 1;
const pendingKernelRequests = new Map<string, PendingKernelRequest>();
const pendingKernelStreams = new Map<string, PendingKernelStream>();
const streamRequestIdsByStreamId = new Map<string, string>();

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
        sendKernelFrame("kernel.capability.cancel", { stream_id: streamId, invocation_id: streamId, session_id: `subprocess_reverse_${streamId}` });
      },
    };
  },
};

function handleKernelInbound(frame: JsonRpcRequest): boolean {
  if (typeof frame.id !== "string" || !frame.id.startsWith("kreq-")) return false;
  const requestId = frame.id;

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
      case "kernel/stream.chunk":
      case "stream.chunk":
        pendingStream.callbacks.onChunk(frame.data);
        return true;
      case "kernel/stream.ended":
      case "stream.ended":
        pendingKernelStreams.delete(requestId);
        if (pendingStream.streamId) streamRequestIdsByStreamId.delete(pendingStream.streamId);
        pendingStream.callbacks.onEnd?.(frame.summary);
        return true;
      case "kernel/stream.error":
      case "stream.error":
        pendingKernelStreams.delete(requestId);
        if (pendingStream.streamId) streamRequestIdsByStreamId.delete(pendingStream.streamId);
        pendingStream.callbacks.onError?.(frame.error);
        return true;
      case "kernel/stream.cancelled":
      case "stream.cancelled":
        pendingKernelStreams.delete(requestId);
        if (pendingStream.streamId) streamRequestIdsByStreamId.delete(pendingStream.streamId);
        pendingStream.callbacks.onCancelled?.();
        return true;
      case "kernel/stream.timeout":
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
