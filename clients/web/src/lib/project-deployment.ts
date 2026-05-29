export interface DockerDeploymentDescriptor {
  image: string;
  container_port: number;
  port_name: string;
  route_id: string;
  health_path?: string;
  pull_if_missing: boolean;
}

export interface DockerDeploymentParseResult {
  descriptor: DockerDeploymentDescriptor | null;
  error?: string;
}

// Explicit web deploy metadata lives at project.metadata.deployment.docker.
// This intentionally accepts only one prebuilt Docker HTTP image and host-owned
// port/proxy metadata; env, volumes, mounts, and secrets are not part of Phase 5.
export function parseDockerDeploymentDescriptor(
  projectId: string,
  metadata: unknown,
): DockerDeploymentParseResult {
  const docker = readDockerDeploymentMetadata(metadata);
  if (docker === undefined) return { descriptor: null };
  if (!isRecord(docker)) return { descriptor: null, error: "deployment.docker must be an object" };

  if ("env" in docker || "environment" in docker || "secrets" in docker || "volumes" in docker || "mounts" in docker || "binds" in docker) {
    return { descriptor: null, error: "deployment.docker cannot declare env, secrets, volumes, mounts, or binds" };
  }

  const image = docker.image;
  if (typeof image !== "string" || !isSafeDockerImage(image.trim())) {
    return { descriptor: null, error: "deployment.docker.image must be a safe Docker image reference" };
  }

  const rawContainerPort = docker.container_port;
  if (typeof rawContainerPort !== "number" || !Number.isInteger(rawContainerPort) || rawContainerPort < 1 || rawContainerPort > 65_535) {
    return { descriptor: null, error: "deployment.docker.container_port must be an integer in 1..65535" };
  }
  const containerPort = rawContainerPort;

  const portName = docker.port_name === undefined ? "web" : docker.port_name;
  if (typeof portName !== "string" || !isSafeRouteToken(portName)) {
    return { descriptor: null, error: "deployment.docker.port_name must be label-safe" };
  }

  const routeId = docker.route_id === undefined ? `${projectId}-web` : docker.route_id;
  if (typeof routeId !== "string" || !isSafeRouteToken(routeId)) {
    return { descriptor: null, error: "deployment.docker.route_id must be label-safe" };
  }

  const healthPath = docker.health_path;
  if (healthPath !== undefined && (typeof healthPath !== "string" || !healthPath.startsWith("/") || healthPath.length > 256)) {
    return { descriptor: null, error: "deployment.docker.health_path must start with /" };
  }

  const pullIfMissing = docker.pull_if_missing ?? false;
  if (typeof pullIfMissing !== "boolean") {
    return { descriptor: null, error: "deployment.docker.pull_if_missing must be a boolean" };
  }

  return {
    descriptor: {
      image: image.trim(),
      container_port: containerPort,
      port_name: portName,
      route_id: routeId,
      ...(healthPath ? { health_path: healthPath } : {}),
      pull_if_missing: pullIfMissing,
    },
  };
}

function readDockerDeploymentMetadata(metadata: unknown): unknown {
  if (!isRecord(metadata)) return undefined;
  const deployment = metadata.deployment;
  if (!isRecord(deployment)) return undefined;
  return deployment.docker;
}

function isRecord(value: unknown): value is Record<string, unknown> {
  return typeof value === "object" && value !== null && !Array.isArray(value);
}

function isSafeDockerImage(image: string): boolean {
  return image.length > 0
    && image.length <= 255
    && !image.startsWith("-")
    && !image.includes("..")
    && /^[A-Za-z0-9./:_@-]+$/.test(image);
}

function isSafeRouteToken(value: string): boolean {
  return value.length > 0
    && value.length <= 128
    && !value.includes("..")
    && /^[A-Za-z0-9][A-Za-z0-9._-]*$/.test(value);
}
