import { parseDockerDeploymentDescriptor } from "./project-deployment";

function assertEqual<T>(actual: T, expected: T) {
  if (actual !== expected) {
    throw new Error(`expected ${String(expected)}, got ${String(actual)}`);
  }
}

function assertDeepEqual(actual: unknown, expected: unknown) {
  const actualJson = JSON.stringify(actual);
  const expectedJson = JSON.stringify(expected);
  if (actualJson !== expectedJson) {
    throw new Error(`expected ${expectedJson}, got ${actualJson}`);
  }
}

assertDeepEqual(parseDockerDeploymentDescriptor("project-1", {}), { descriptor: null });

assertDeepEqual(parseDockerDeploymentDescriptor("project-1", {
  deployment: {
    docker: {
      image: "ghcr.io/example/app:1.2.3",
      container_port: 8080,
    },
  },
}), {
  descriptor: {
    image: "ghcr.io/example/app:1.2.3",
    container_port: 8080,
    port_name: "web",
    route_id: "project-1-web",
    pull_if_missing: false,
  },
});

assertDeepEqual(parseDockerDeploymentDescriptor("project-1", {
  deployment: {
    docker: {
      image: "example/app@sha256:abc123",
      container_port: 80,
      port_name: "http",
      route_id: "project-1-http",
      health_path: "/healthz",
      pull_if_missing: true,
    },
  },
}).descriptor, {
  image: "example/app@sha256:abc123",
  container_port: 80,
  port_name: "http",
  route_id: "project-1-http",
  health_path: "/healthz",
  pull_if_missing: true,
});

assertEqual(Boolean(parseDockerDeploymentDescriptor("project-1", {
  deployment: { docker: { image: "bad image", container_port: 80 } },
}).error), true);

assertEqual(Boolean(parseDockerDeploymentDescriptor("project-1", {
  deployment: { docker: { image: "example/app:latest", container_port: 0 } },
}).error), true);

assertEqual(Boolean(parseDockerDeploymentDescriptor("project-1", {
  deployment: { docker: { image: "example/app:latest", container_port: 80, route_id: "../bad" } },
}).error), true);

assertEqual(Boolean(parseDockerDeploymentDescriptor("project-1", {
  deployment: { docker: { image: "example/app:latest", container_port: 80, env: { API_KEY: "nope" } } },
}).error), true);
