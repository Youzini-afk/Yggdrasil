import readline from "node:readline";

export type JsonValue = null | boolean | number | string | JsonValue[] | { [key: string]: JsonValue };

export interface JsonRpcRequest {
  jsonrpc?: "2.0";
  id?: string | number | null;
  method?: string;
  params?: Record<string, JsonValue>;
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
export type HandshakeHandler = (params: HandshakeParams) => JsonValue | Promise<JsonValue>;

export interface SubprocessPackageOptions {
  onHandshake?: HandshakeHandler;
  onInvoke: CapabilityHandler;
}

function respond(id: JsonRpcRequest["id"], payload: Record<string, JsonValue>) {
  process.stdout.write(JSON.stringify({ jsonrpc: "2.0", id, ...payload }) + "\n");
}

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

    try {
      if (request.method === "package.handshake") {
        const result = options.onHandshake
          ? await options.onHandshake((request.params ?? {}) as HandshakeParams)
          : { ready: true, package_protocol_version: "0.1.0" };
        respond(request.id, { result: result as JsonValue });
      } else if (request.method === "capability.invoke") {
        const output = await options.onInvoke((request.params ?? {}) as unknown as CapabilityInvokeParams);
        respond(request.id, { result: { output } as JsonValue });
      } else {
        respond(request.id, { error: { code: "unknown_method", message: request.method ?? "<missing>" } as JsonValue });
      }
    } catch (error) {
      respond(request.id, { error: { code: "package_error", message: String(error) } as JsonValue });
    }
  });
}
