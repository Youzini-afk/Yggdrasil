export interface KernelTransport {
  invoke(method: string, params: unknown): Promise<unknown>;
  invokeStream(method: string, params: unknown): AsyncIterable<unknown>;
  close?(): Promise<void>;
}

export class KernelClient {
  constructor(public transport: KernelTransport) {}
}

export function fromHttpRpc(url: string): KernelClient {
  let nextId = 1;
  const transport: KernelTransport = {
    async invoke(method: string, params: unknown): Promise<unknown> {
      const response = await fetch(url, {
        method: "POST",
        headers: { "content-type": "application/json" },
        body: JSON.stringify({ jsonrpc: "2.0", id: nextId++, method, params }),
      });
      if (!response.ok) {
        throw new Error(`Yggdrasil RPC ${method} failed with HTTP ${response.status}`);
      }
      const envelope = (await response.json()) as { result?: unknown; error?: unknown };
      if (envelope.error !== undefined) {
        throw new Error(`Yggdrasil RPC ${method} failed: ${JSON.stringify(envelope.error)}`);
      }
      return envelope.result;
    },
    async *invokeStream(method: string, params: unknown): AsyncIterable<unknown> {
      yield await this.invoke(method, params);
    },
  };
  return new KernelClient(transport);
}

export function fromStdio(stream: NodeJS.ReadWriteStream): KernelClient {
  let nextId = 1;
  let buffer = "";
  const pending = new Map<number, { resolve: (value: unknown) => void; reject: (error: Error) => void }>();

  stream.on("data", (chunk: Buffer | string) => {
    buffer += chunk.toString();
    for (;;) {
      const newline = buffer.indexOf("\n");
      if (newline < 0) break;
      const line = buffer.slice(0, newline).trim();
      buffer = buffer.slice(newline + 1);
      if (line.length === 0) continue;
      const message = JSON.parse(line) as { id?: number; result?: unknown; error?: unknown };
      if (typeof message.id !== "number") continue;
      const waiter = pending.get(message.id);
      if (!waiter) continue;
      pending.delete(message.id);
      if (message.error !== undefined) {
        waiter.reject(new Error(JSON.stringify(message.error)));
      } else {
        waiter.resolve(message.result);
      }
    }
  });

  const transport: KernelTransport = {
    invoke(method: string, params: unknown): Promise<unknown> {
      const id = nextId++;
      const request = { jsonrpc: "2.0", id, method, params };
      return new Promise((resolve, reject) => {
        pending.set(id, { resolve, reject });
        stream.write(`${JSON.stringify(request)}\n`);
      });
    },
    async *invokeStream(method: string, params: unknown): AsyncIterable<unknown> {
      yield await this.invoke(method, params);
    },
    async close(): Promise<void> {
      stream.end();
    },
  };
  return new KernelClient(transport);
}
