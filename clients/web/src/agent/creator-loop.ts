// ========================================================================
// Experience Beta 5 — Forge Creator Loop UI
// ========================================================================
// Public-protocol-only panels for the creator authoring workflow scaffold.
// Gives creators package inventory, experience descriptor preview, fixture
// controls, diagnostics explainability, and template-to-playable guidance.
//
// All data is derived from public protocol types (packages, capabilities,
// surfaces, events, proposals, assets, projections). No runtime internals,
// no SQLite, no privileged Studio, no marketplace/monetization UI.
// Chat-first patterns are deliberately excluded.
// ========================================================================

import type {
  AssetRecord,
  KernelEvent,
  PackageRecord,
  ProjectionRecord,
  ProposalRecord,
  RegisteredCapability,
  SurfaceContributionRecord,
} from "../protocol/client";
import { escapeHtml, formatJson } from "../utils/html";

// ========================================================================
// Types — View Models for the Creator Loop
// ========================================================================

/** Top-level creator loop view model */
export interface CreatorLoopModel {
  creatorReadiness: CreatorReadiness;
  templateRecommendations: TemplateRecommendation[];
  packageDiagnostics: PackageDiagnosticCard[];
  compositionReadiness: CompositionReadiness[];
  fixtureControls: FixtureControl[];
  replacementChecklist: ReplacementChecklistItem[];
  walkthroughSteps: WalkthroughStep[];
}

/** Overall creator readiness state */
export interface CreatorReadiness {
  overall: "ready" | "almost_ready" | "needs_work" | "unknown";
  packageCount: number;
  capabilityCount: number;
  surfaceCount: number;
  experienceEntryCount: number;
  sessionActive: boolean;
  missingPieces: string[];
  recommendations: string[];
}

/** A template recommendation card for the "Template to Playable" panel */
export interface TemplateRecommendation {
  id: string;
  label: string;
  description: string;
  templateType: string;
  suggestedFor: string;
  commands: string[];
  prerequisites: string[];
}

/** Per-package diagnostic card explaining why a package is in its current state */
export interface PackageDiagnosticCard {
  packageId: string;
  state: string;
  entryKind: string;
  issueCount: number;
  issues: DiagnosticIssue[];
  strengthCount: number;
  strengths: string[];
  summary: string;
}

export interface DiagnosticIssue {
  severity: "error" | "warn" | "info";
  message: string;
  code?: string;
}

/** Composition readiness state — whether the package set is structurally sound */
export interface CompositionReadiness {
  compositionId: string;
  status: "valid" | "incomplete" | "invalid" | "unknown";
  packageIds: string[];
  missingPackages: string[];
  surfaceSlots: string[];
  uncoveredSlots: string[];
  checkResult?: string;
}

/** Fixture / reload control with public-protocol payload preview only */
export interface FixtureControl {
  id: string;
  label: string;
  description: string;
  packageId: string;
  action: "check" | "conformance" | "run-fixture" | "reload";
  disabled: boolean;
  disabledReason: string;
  payloadPreview: string;
}

/** Replacement & permission checklist item */
export interface ReplacementChecklistItem {
  category: string;
  label: string;
  status: "ok" | "warn" | "error" | "info";
  detail: string;
  actionNeeded?: string;
}

/** A step in the "template to playable" walkthrough */
export interface WalkthroughStep {
  order: number;
  title: string;
  description: string;
  status: "complete" | "in_progress" | "pending" | "skipped";
  action?: string;
  actionCommand?: string;
}

// ========================================================================
// Heuristic helpers (public protocol strings only)
// ========================================================================

const EXPERIENCE_ENTRY_SLOT = "experience_entry";
const FORGE_PANEL_SLOT = "forge_panel";
const ASSISTANT_ACTION_SLOT = "assistant_action";

/** Template kinds that the Ygg CLI supports */
const TEMPLATE_KINDS: TemplateRecommendation[] = [
  {
    id: "basic",
    label: "Basic Package",
    description: "A minimal subprocess package with a single capability. Best starting point for prototyping a new capability.",
    templateType: "basic",
    suggestedFor: "New capabilities, prototyping, simple tool wrappers",
    commands: [
      "cargo run -p ygg-cli -- init-package ./my-package --id example/my-package --entry subprocess --language python",
      "cargo run -p ygg-cli -- init-package ./my-package --id example/my-package --entry subprocess --language typescript",
    ],
    prerequisites: ["manifest.yaml", "entry script (subprocess)"],
  },
  {
    id: "experience",
    label: "Experience Package",
    description: "A full experience package with an experience_entry surface, capabilities, and session lifecycle support. Required for Play-launchable experiences.",
    templateType: "experience",
    suggestedFor: "Play-launchable experiences, content-driven packages, interactive sessions",
    commands: [
      "cargo run -p ygg-cli -- init-package ./my-experience --id example/my-experience --entry subprocess --language typescript-experience",
    ],
    prerequisites: ["experience_entry surface", "launch capability", "session template", "permission requirements"],
  },
  {
    id: "play-renderer",
    label: "Play Renderer",
    description: "Renders content inside the Play surface. Useful for custom visualizations, media playback, or rich UI in the launcher.",
    templateType: "play-renderer",
    suggestedFor: "Custom launcher UIs, media renderers, visualizations",
    commands: [
      "cargo run -p ygg-cli -- init-package ./my-renderer --id example/my-renderer --entry subprocess --language typescript",
    ],
    prerequisites: ["play_renderer surface slot", "render capability"],
  },
  {
    id: "forge-panel",
    label: "Forge Panel",
    description: "Contributes a panel to the Forge surface. Best for authoring tools, inspectors, and package-specific debug views.",
    templateType: "forge-panel",
    suggestedFor: "Authoring tools, debug inspectors, package-specific editors",
    commands: [
      "cargo run -p ygg-cli -- init-package ./my-forge-panel --id example/my-forge-panel --entry subprocess --language typescript",
    ],
    prerequisites: ["forge_panel surface slot", "Forge section capability"],
  },
  {
    id: "assistant-action",
    label: "Assistant Action",
    description: "Registers an action in the Assistant Drawer. Good for quick edits, approvals, or simple tool invocations from the assist panel.",
    templateType: "assistant-action",
    suggestedFor: "Quick edit actions, one-shot tool invocations, light approvals",
    commands: [
      "cargo run -p ygg-cli -- init-package ./my-action --id example/my-action --entry subprocess --language typescript",
    ],
    prerequisites: ["assistant_action surface slot", "action handler capability"],
  },
  {
    id: "full-surface",
    label: "Full Surface Package",
    description: "A package that contributes multiple surfaces (experience_entry, forge_panel, assistant_action) with full capability and permission sets.",
    templateType: "full-surface",
    suggestedFor: "Complex packages that need multiple surface slots, comprehensive authoring tools",
    commands: [
      "cargo run -p ygg-cli -- init-package ./my-full-pkg --id example/my-full-pkg --entry subprocess --language typescript",
    ],
    prerequisites: ["multiple surface slots", "cross-slot capability routing", "permission requirements per slot"],
  },
];

/** The walkthrough steps for going from template to playable */
const DEFAULT_WALKTHROUGH_STEPS: Omit<WalkthroughStep, "status">[] = [
  {
    order: 1,
    title: "Initialize Package",
    description: "Create a new package from a template using the Ygg CLI. Choose the template type that matches your use case.",
    action: "cli-init",
    actionCommand: "cargo run -p ygg-cli -- init-package ./my-package --id example/my-package --entry subprocess --language typescript",
  },
  {
    order: 2,
    title: "Declare Capabilities",
    description: "Add capability declarations to your manifest.yaml. Capabilities define what your package can do — each one maps to a named, versioned function.",
    action: "edit-manifest",
  },
  {
    order: 3,
    title: "Register Surfaces",
    description: "Declare surface contributions in your manifest.yaml. Surfaces define where your package appears (experience_entry, forge_panel, assistant_action, etc.) and what permissions they need.",
    action: "edit-manifest",
  },
  {
    order: 4,
    title: "Define Permissions",
    description: "Specify required permissions for each surface contribution. Permission requirements are declared with a risk level (low/medium/high) and an approval policy.",
    action: "edit-manifest",
  },
  {
    order: 5,
    title: "Package Check",
    description: "Run the package check command to validate your manifest.yaml against the schema. Fix any errors before proceeding.",
    action: "cli-check",
    actionCommand: "cargo run -p ygg-cli -- package check ./my-package/manifest.yaml",
  },
  {
    order: 6,
    title: "Conformance Test",
    description: "Run conformance tests to verify your package adheres to Yggdrasil protocol expectations. This ensures your package can be loaded and discovered correctly.",
    action: "cli-conformance",
    actionCommand: "cargo run -p ygg-cli -- package conformance ./my-package/manifest.yaml",
  },
  {
    order: 7,
    title: "Load & Verify",
    description: "Load your package into the running host and verify it appears in the Package Inventory. Check that capabilities, surfaces, and permissions are registered correctly.",
    action: "cli-reload",
    actionCommand: "cargo run -p ygg-cli -- package reload ./my-package/manifest.yaml",
  },
  {
    order: 8,
    title: "Launch Experience",
    description: "If your package declares an experience_entry surface, launch it from the Play surface. Verify the session opens and events flow correctly.",
    action: "launch-play",
  },
];

// ========================================================================
// Builder — produces CreatorLoopModel from public protocol data
// ========================================================================

export function buildCreatorLoopModel(
  packages: PackageRecord[],
  capabilities: RegisteredCapability[],
  allSurfaces: SurfaceContributionRecord[],
  events: KernelEvent[],
  proposals: ProposalRecord[],
  assets: AssetRecord[],
  projections: ProjectionRecord[],
  sessionId?: string,
): CreatorLoopModel {
  const creatorReadiness = buildCreatorReadiness(packages, capabilities, allSurfaces, events, sessionId);
  const templateRecommendations = buildTemplateRecommendations(packages, allSurfaces);
  const packageDiagnostics = buildPackageDiagnostics(packages, capabilities, allSurfaces);
  const compositionReadiness = buildCompositionReadiness(packages, capabilities, allSurfaces);
  const fixtureControls = buildFixtureControls(packages, capabilities);
  const replacementChecklist = buildReplacementChecklist(packages, capabilities, allSurfaces, events, proposals, assets, projections);
  const walkthroughSteps = buildWalkthroughSteps(packages, capabilities, allSurfaces, events);

  return {
    creatorReadiness,
    templateRecommendations,
    packageDiagnostics,
    compositionReadiness,
    fixtureControls,
    replacementChecklist,
    walkthroughSteps,
  };
}

function buildCreatorReadiness(
  packages: PackageRecord[],
  capabilities: RegisteredCapability[],
  allSurfaces: SurfaceContributionRecord[],
  events: KernelEvent[],
  sessionId?: string,
): CreatorReadiness {
  const experienceEntryCount = allSurfaces.filter((s) => s.surface.slot === EXPERIENCE_ENTRY_SLOT).length;
  const sessionActive = !!sessionId && events.length > 0;
  const missingPieces: string[] = [];
  const recommendations: string[] = [];

  if (packages.length === 0) {
    missingPieces.push("No packages loaded");
    recommendations.push("Initialize a package with `cargo run -p ygg-cli -- init-package`");
  }
  if (capabilities.length === 0) {
    missingPieces.push("No capabilities discovered");
    recommendations.push("Packages must declare at least one capability in their manifest.yaml");
  }
  if (allSurfaces.length === 0) {
    missingPieces.push("No surface contributions");
    recommendations.push("Add surface contributions to your package manifest.yaml to appear in the shell");
  }
  if (experienceEntryCount === 0) {
    missingPieces.push("No experience_entry surface found");
    recommendations.push("Add an experience_entry surface to make your package launchable from Play");
  }
  if (!sessionActive && packages.length > 0) {
    recommendations.push("Open a session to begin receiving events and testing your package");
  }

  let overall: CreatorReadiness["overall"];
  if (packages.length > 0 && capabilities.length > 0 && experienceEntryCount > 0) {
    overall = sessionActive ? "ready" : "almost_ready";
  } else if (packages.length > 0 || capabilities.length > 0) {
    overall = "needs_work";
  } else {
    overall = "unknown";
  }

  return {
    overall,
    packageCount: packages.length,
    capabilityCount: capabilities.length,
    surfaceCount: allSurfaces.length,
    experienceEntryCount,
    sessionActive,
    missingPieces,
    recommendations,
  };
}

function buildTemplateRecommendations(
  packages: PackageRecord[],
  allSurfaces: SurfaceContributionRecord[],
): TemplateRecommendation[] {
  const existingSlots = new Set(allSurfaces.map((s) => s.surface.slot));
  const existingTemplates = new Set(packages.map((p) => p.entry_kind));

  // Filter recommendations based on what's missing
  const recommendations = TEMPLATE_KINDS.filter((t) => {
    // Skip templates already present
    if (t.id === "experience" && existingSlots.has(EXPERIENCE_ENTRY_SLOT)) return false;
    if (t.id === "forge-panel" && existingSlots.has(FORGE_PANEL_SLOT)) return false;
    if (t.id === "assistant-action" && existingSlots.has(ASSISTANT_ACTION_SLOT)) return false;
    if (t.id === "basic" && existingTemplates.has("subprocess")) return false;
    return true;
  });

  return recommendations;
}

function buildPackageDiagnostics(
  packages: PackageRecord[],
  capabilities: RegisteredCapability[],
  allSurfaces: SurfaceContributionRecord[],
): PackageDiagnosticCard[] {
  const capsByPackage = new Map<string, RegisteredCapability[]>();
  for (const cap of capabilities) {
    const list = capsByPackage.get(cap.provider_package_id) ?? [];
    list.push(cap);
    capsByPackage.set(cap.provider_package_id, list);
  }

  const surfacesByPackage = new Map<string, SurfaceContributionRecord[]>();
  for (const s of allSurfaces) {
    const list = surfacesByPackage.get(s.package_id) ?? [];
    list.push(s);
    surfacesByPackage.set(s.package_id, list);
  }

  return packages.map((pkg) => {
    const issues: DiagnosticIssue[] = [];
    const strengths: string[] = [];

    // State-based diagnostics
    if (pkg.state === "error" || pkg.state === "failed") {
      issues.push({
        severity: "error",
        message: `Package is in ${pkg.state} state`,
        code: "package_error",
      });
    } else if (pkg.state === "loaded") {
      strengths.push("Package loaded successfully");
    } else if (pkg.state === "activated") {
      strengths.push("Package activated and ready");
    }

    // Capability diagnostics
    const pkgCaps = capsByPackage.get(pkg.id) ?? [];
    if (pkgCaps.length === 0) {
      issues.push({
        severity: "warn",
        message: "No capabilities registered — package may not function",
        code: "missing_capability",
      });
    } else {
      strengths.push(`${pkgCaps.length} capabilit${pkgCaps.length === 1 ? "y" : "ies"} registered`);
    }

    // Surface diagnostics
    const pkgSurfaces = surfacesByPackage.get(pkg.id) ?? [];
    if (pkgSurfaces.length === 0) {
      issues.push({
        severity: "warn",
        message: "No surface contributions — package may not appear in shell",
        code: "missing_surface",
      });
    } else {
      strengths.push(`${pkgSurfaces.length} surface${pkgSurfaces.length === 1 ? "" : "s"} contributed`);

      // Check for experience_entry surfaces
      const hasExperienceEntry = pkgSurfaces.some((s) => s.surface.slot === EXPERIENCE_ENTRY_SLOT);
      if (hasExperienceEntry) {
        strengths.push("Declares experience_entry — launchable from Play");
      }

      // Check for permissions
      const highRiskPerms = pkgSurfaces.filter((s) =>
        s.surface.required_permissions.some((p) => p.risk === "high"),
      );
      if (highRiskPerms.length > 0) {
        issues.push({
          severity: "info",
          message: `${highRiskPerms.length} surface${highRiskPerms.length === 1 ? "" : "s"} require${highRiskPerms.length === 1 ? "s" : ""} high-risk permissions`,
          code: "high_risk_permissions",
        });
      }
    }

    // Entry kind diagnostics
    if (pkg.entry_kind === "subprocess" && pkgCaps.length === 0) {
      issues.push({
        severity: "warn",
        message: "Subprocess entry with no capabilities — check manifest.yaml",
        code: "inactive_entry",
      });
    }

    // Build summary
    let summary: string;
    if (pkg.state === "error") {
      summary = `Package ${pkg.id} encountered an error. Check the host logs and manifest.yaml for issues.`;
    } else if (pkgCaps.length === 0 && pkgSurfaces.length === 0) {
      summary = `Package ${pkg.id} is loaded but has no capabilities or surfaces. It may be incomplete.`;
    } else if (pkgCaps.length > 0 && pkgSurfaces.length > 0) {
      summary = `Package ${pkg.id} is operational with ${pkgCaps.length} capabilities and ${pkgSurfaces.length} surfaces.`;
    } else {
      summary = `Package ${pkg.id} is partially configured. Review diagnostics below.`;
    }

    return {
      packageId: pkg.id,
      state: pkg.state,
      entryKind: pkg.entry_kind,
      issueCount: issues.length,
      issues,
      strengthCount: strengths.length,
      strengths,
      summary,
    };
  });
}

function buildCompositionReadiness(
  packages: PackageRecord[],
  capabilities: RegisteredCapability[],
  allSurfaces: SurfaceContributionRecord[],
): CompositionReadiness[] {
  // Build composition readiness from the current package set.
  // We treat the entire loaded package set as one "composition."
  const packageIds = packages.map((p) => p.id);
  const allSlots = [...new Set(allSurfaces.map((s) => s.surface.slot))];

  // Required slots for a minimal playable experience
  const requiredSlots = ["experience_entry"];
  // Recommended slots
  const recommendedSlots = ["forge_panel", "assistant_action"];

  const coveredSlots = new Set(allSlots);
  const uncoveredSlots = requiredSlots.concat(recommendedSlots).filter((s) => !coveredSlots.has(s));

  // Detect missing packages: if a capability references a provider not in the loaded set
  const referencedProviders = new Set(capabilities.map((c) => c.provider_package_id));
  const missingPackages = [...referencedProviders].filter((pid) => !packageIds.includes(pid));

  let status: CompositionReadiness["status"];
  const errors: string[] = [];

  if (packages.length === 0) {
    status = "unknown";
    errors.push("No packages loaded");
  } else {
    const hasRequiredSlot = allSlots.includes(EXPERIENCE_ENTRY_SLOT);
    if (!hasRequiredSlot) {
      errors.push("Missing experience_entry surface — required for Play launch");
    }
    if (missingPackages.length > 0) {
      errors.push(`Missing package${missingPackages.length === 1 ? "" : "s"}: ${missingPackages.join(", ")}`);
    }
    if (capabilities.length === 0) {
      errors.push("No capabilities discovered");
    }
    status = errors.length === 0 ? "valid" : errors.length <= 2 ? "incomplete" : "invalid";
  }

  return [
    {
      compositionId: "workspace-composition",
      status,
      packageIds,
      missingPackages,
      surfaceSlots: allSlots,
      uncoveredSlots,
      checkResult: errors.length > 0 ? errors.join("; ") : "All required slots and capabilities are present.",
    },
  ];
}

function buildFixtureControls(
  packages: PackageRecord[],
  capabilities: RegisteredCapability[],
): FixtureControl[] {
  const controls: FixtureControl[] = [];

  // Per-package fixture controls
  for (const pkg of packages) {
    // Check control
    controls.push({
      id: `check-${pkg.id}`,
      label: `Check ${pkg.id}`,
      description: `Validate the manifest.yaml schema for ${pkg.id}`,
      packageId: pkg.id,
      action: "check",
      disabled: false,
      disabledReason: "",
      payloadPreview: JSON.stringify({
        method: "kernel.v1.package.check",
        params: { package_id: pkg.id },
        expected_response: { valid: true, errors: [], warnings: [] },
      }, null, 2),
    });

    // Conformance control
    controls.push({
      id: `conformance-${pkg.id}`,
      label: `Conformance ${pkg.id}`,
      description: `Run protocol conformance tests for ${pkg.id}`,
      packageId: pkg.id,
      action: "conformance",
      disabled: false,
      disabledReason: "",
      payloadPreview: JSON.stringify({
        method: "kernel.v1.package.conformance",
        params: { package_id: pkg.id },
        expected_response: { conformance: "pass|fail|warn", details: [] },
      }, null, 2),
    });

    // Fixture run control (available when package has capabilities)
    if (capabilities.some((c) => c.provider_package_id === pkg.id)) {
      controls.push({
        id: `fixture-${pkg.id}`,
        label: `Run fixture ${pkg.id}`,
        description: `Execute a fixture test for ${pkg.id}`,
        packageId: pkg.id,
        action: "run-fixture",
        disabled: false,
        disabledReason: "",
        payloadPreview: JSON.stringify({
          method: "kernel.v1.package.run_fixture",
          params: { package_id: pkg.id },
          expected_response: { fixture_result: "pass|fail", events: [] },
        }, null, 2),
      });
    }

    // Reload control
    controls.push({
      id: `reload-${pkg.id}`,
      label: `Reload ${pkg.id}`,
      description: `Reload package ${pkg.id} to pick up manifest changes`,
      packageId: pkg.id,
      action: "reload",
      disabled: false,
      disabledReason: "",
      payloadPreview: JSON.stringify({
        method: "kernel.v1.package.reload",
        params: { package_id: pkg.id },
        expected_response: { state: "loaded|activated|error", error: null },
      }, null, 2),
    });
  }

  // Add a disabled-safe stub for when no packages are loaded
  if (packages.length === 0) {
    controls.push({
      id: "check-stub",
      label: "Check package (disabled-safe)",
      description: "Validate a package manifest.yaml — no target package loaded",
      packageId: "__stub__",
      action: "check",
      disabled: true,
      disabledReason: "No packages loaded. Initialize a package first.",
      payloadPreview: JSON.stringify({
        method: "kernel.v1.package.check",
        params: { package_id: "<package-id>" },
        expected_response: { valid: true, errors: [], warnings: [] },
      }, null, 2),
    });
    controls.push({
      id: "reload-stub",
      label: "Reload package (disabled-safe)",
      description: "Re-read a package manifest from disk — no target package loaded",
      packageId: "__stub__",
      action: "reload",
      disabled: true,
      disabledReason: "No packages to reload. Load a package first.",
      payloadPreview: JSON.stringify({
        method: "kernel.v1.package.reload",
        params: { package_id: "<package-id>" },
        expected_response: { state: "loaded|activated|error", error: null },
      }, null, 2),
    });
  }

  return controls;
}

function buildReplacementChecklist(
  packages: PackageRecord[],
  capabilities: RegisteredCapability[],
  allSurfaces: SurfaceContributionRecord[],
  events: KernelEvent[],
  proposals: ProposalRecord[],
  assets: AssetRecord[],
  projections: ProjectionRecord[],
): ReplacementChecklistItem[] {
  const items: ReplacementChecklistItem[] = [];

  // Category: Package Health
  const packagesWithErrors = packages.filter((p) => p.state === "error" || p.state === "failed");
  const packagesWithoutCaps = packages.filter((p) => p.capability_count === 0);
  const packagesWithoutSurfaces = packages.filter((p) =>
    allSurfaces.filter((s) => s.package_id === p.id).length === 0,
  );

  items.push({
    category: "Package Health",
    label: "All packages loaded without errors",
    status: packagesWithErrors.length === 0 ? "ok" : "error",
    detail: packagesWithErrors.length === 0
      ? "All packages are in a healthy state."
      : `${packagesWithErrors.length} package${packagesWithErrors.length === 1 ? "" : "s"} in error state.`,
    actionNeeded: packagesWithErrors.length > 0
      ? "Check host logs and review manifest.yaml for errors."
      : undefined,
  });

  items.push({
    category: "Package Health",
    label: "All packages declare capabilities",
    status: packagesWithoutCaps.length === 0 ? "ok" : "warn",
    detail: packagesWithoutCaps.length === 0
      ? "Every loaded package declares at least one capability."
      : `${packagesWithoutCaps.length} package${packagesWithoutCaps.length === 1 ? "" : "s"} with no capabilities.`,
    actionNeeded: packagesWithoutCaps.length > 0
      ? "Add capability declarations to these packages' manifest.yaml files."
      : undefined,
  });

  items.push({
    category: "Package Health",
    label: "All packages contribute surfaces",
    status: packagesWithoutSurfaces.length === 0 ? "ok" : "warn",
    detail: packagesWithoutSurfaces.length === 0
      ? "Every loaded package contributes at least one surface."
      : `${packagesWithoutSurfaces.length} package${packagesWithoutSurfaces.length === 1 ? "" : "s"} with no surface contributions.`,
    actionNeeded: packagesWithoutSurfaces.length > 0
      ? "Add surface contributions to these packages to make them visible in the shell."
      : undefined,
  });

  // Category: Surface Coverage
  const experienceEntries = allSurfaces.filter((s) => s.surface.slot === EXPERIENCE_ENTRY_SLOT);
  const forgePanels = allSurfaces.filter((s) => s.surface.slot === FORGE_PANEL_SLOT);
  const assistantActions = allSurfaces.filter((s) => s.surface.slot === ASSISTANT_ACTION_SLOT);

  items.push({
    category: "Surface Coverage",
    label: "Experience entry surface available",
    status: experienceEntries.length > 0 ? "ok" : "error",
    detail: experienceEntries.length > 0
      ? `${experienceEntries.length} experience entr${experienceEntries.length === 1 ? "y" : "ies"} ready for Play launch.`
      : "No experience_entry surface found. Add a package with an experience surface.",
    actionNeeded: experienceEntries.length === 0
      ? "Use `init-package --entry subprocess --language typescript-experience` to create one."
      : undefined,
  });

  items.push({
    category: "Surface Coverage",
    label: "Forge panels contributed by packages",
    status: forgePanels.length > 0 ? "ok" : "info",
    detail: forgePanels.length > 0
      ? `${forgePanels.length} forge panel${forgePanels.length === 1 ? "" : "s"} contributed.`
      : "No forge_panel surfaces — packages may not have dedicated authoring UIs.",
    actionNeeded: forgePanels.length === 0
      ? "Package authors can add forge_panel surfaces for custom authoring tools."
      : undefined,
  });

  items.push({
    category: "Surface Coverage",
    label: "Assistant actions registered",
    status: assistantActions.length > 0 ? "ok" : "info",
    detail: assistantActions.length > 0
      ? `${assistantActions.length} assistant action${assistantActions.length === 1 ? "" : "s"} registered.`
      : "No assistant_action surfaces — quick action buttons won't appear in the Assistant Drawer.",
    actionNeeded: assistantActions.length === 0
      ? "Package authors can add assistant_action surfaces for quick operations."
      : undefined,
  });

  // Category: Permissions
  const highRiskSurfaces = allSurfaces.filter((s) =>
    s.surface.required_permissions.some((p) => p.risk === "high"),
  );
  const surfacesWithApproval = allSurfaces.filter((s) =>
    s.surface.approval_policy && s.surface.approval_policy !== "none",
  );

  items.push({
    category: "Permissions",
    label: "High-risk permissions reviewed",
    status: highRiskSurfaces.length === 0 ? "ok" : "warn",
    detail: highRiskSurfaces.length === 0
      ? "No high-risk permission requirements detected."
      : `${highRiskSurfaces.length} surface${highRiskSurfaces.length === 1 ? "" : "s"} require${highRiskSurfaces.length === 1 ? "s" : ""} high-risk permissions.`,
    actionNeeded: highRiskSurfaces.length > 0
      ? "Review high-risk permission requirements and ensure they are justified."
      : undefined,
  });

  items.push({
    category: "Permissions",
    label: "Approval policies defined",
    status: surfacesWithApproval.length > 0 ? "ok" : "info",
    detail: surfacesWithApproval.length > 0
      ? `${surfacesWithApproval.length} surface${surfacesWithApproval.length === 1 ? "" : "s"} with non-none approval polic${surfacesWithApproval.length === 1 ? "y" : "ies"}.`
      : "No surfaces define approval policies — all actions are unapproved.",
    actionNeeded: surfacesWithApproval.length === 0
      ? "Consider adding approval policies for surfaces that modify state."
      : undefined,
  });

  // Category: Session & Events
  items.push({
    category: "Session & Events",
    label: "Active session with events flowing",
    status: events.length > 0 ? "ok" : "info",
    detail: events.length > 0
      ? `${events.length} event${events.length === 1 ? "" : "s"} in the current session.`
      : "No events yet. Open a session and interact with packages to generate events.",
    actionNeeded: events.length === 0
      ? "Open a session with 'Begin Experience Session' in the Forge header."
      : undefined,
  });

  items.push({
    category: "Session & Events",
    label: "Proposal-driven change tracking active",
    status: proposals.length > 0 ? "ok" : "info",
    detail: proposals.length > 0
      ? `${proposals.length} proposal${proposals.length === 1 ? "" : "s"} recorded.`
      : "No proposals yet. Proposals are the authoritative record of change in Yggdrasil.",
    actionNeeded: proposals.length === 0
      ? "Load a package that emits proposals, or approve a created proposal."
      : undefined,
  });

  // Category: Assets & Projections
  items.push({
    category: "Assets & Projections",
    label: "Assets declared by packages",
    status: assets.length > 0 ? "ok" : "info",
    detail: assets.length > 0
      ? `${assets.length} asset${assets.length === 1 ? "" : "s"} declared.`
      : "No assets declared. Packages can declare assets for use by surfaces and capabilities.",
    actionNeeded: assets.length === 0
      ? "Packages can declare assets in their manifest.yaml under the assets section."
      : undefined,
  });

  items.push({
    category: "Assets & Projections",
    label: "Projections available",
    status: projections.length > 0 ? "ok" : "info",
    detail: projections.length > 0
      ? `${projections.length} projection${projections.length === 1 ? "" : "s"} available.`
      : "No projections. Projections store derived state snapshots from session events.",
    actionNeeded: projections.length === 0
      ? "Projections are created by package-owned event processing."
      : undefined,
  });

  return items;
}

function buildWalkthroughSteps(
  packages: PackageRecord[],
  capabilities: RegisteredCapability[],
  allSurfaces: SurfaceContributionRecord[],
  events: KernelEvent[],
): WalkthroughStep[] {
  const hasPackages = packages.length > 0;
  const hasCaps = capabilities.length > 0;
  const hasSurfaces = allSurfaces.length > 0;
  const hasExperienceEntry = allSurfaces.some((s) => s.surface.slot === EXPERIENCE_ENTRY_SLOT);
  const hasEvents = events.length > 0;

  return DEFAULT_WALKTHROUGH_STEPS.map((step) => {
    let status: WalkthroughStep["status"];

    switch (step.order) {
      case 1:
        status = hasPackages ? "complete" : "in_progress";
        break;
      case 2:
        status = hasCaps ? "complete" : hasPackages ? "pending" : "pending";
        break;
      case 3:
        status = hasSurfaces ? "complete" : hasCaps ? "pending" : "pending";
        break;
      case 4:
        status = hasSurfaces ? "complete" : "pending";
        break;
      case 5:
        status = hasPackages ? "in_progress" : "pending";
        break;
      case 6:
        status = hasCaps ? "in_progress" : "pending";
        break;
      case 7:
        status = hasPackages ? "in_progress" : "pending";
        break;
      case 8:
        status = hasExperienceEntry && hasEvents ? "complete" : hasExperienceEntry ? "in_progress" : "pending";
        break;
      default:
        status = "pending";
    }

    return { ...step, status };
  });
}

// ========================================================================
// Rendering — HTML string panels for the Forge surface
// ========================================================================

/** Render the full Creator Loop (Beta 5) section */
export function renderCreatorLoopSection(model: CreatorLoopModel): string {
  return `
    <div class="forge-section creator-loop-section">
      <div class="section-header">
        <h2>Creator Loop <span class="phase-badge">Beta 5</span></h2>
        <span class="section-meta">package inventory · diagnostics · fixtures · permissions</span>
      </div>

      <p class="workspace-note">
        Creator Loop panels guide you through the authoring workflow: assess readiness,
        inspect package diagnostics, verify composition structure, run fixture previews,
        and track the template-to-playable journey. All data is derived from public
        protocol samples — no runtime internals, no privileged Studio, no marketplace.
      </p>

      <div class="cloop-grid">
        ${renderCreatorReadinessPanel(model.creatorReadiness)}
        ${renderTemplateToPlayablePanel(model.walkthroughSteps, model.templateRecommendations)}
        ${renderPackageDiagnosticsPanel(model.packageDiagnostics)}
        ${renderCompositionReadinessPanel(model.compositionReadiness)}
        ${renderFixtureControlsPanel(model.fixtureControls)}
        ${renderReplacementChecklistPanel(model.replacementChecklist)}
      </div>
    </div>
  `;
}

// ========================================================================
// Panel: Creator Readiness
// ========================================================================

function renderCreatorReadinessPanel(readiness: CreatorReadiness): string {
  const overallClass = readiness.overall === "ready" ? "severity-ok"
    : readiness.overall === "almost_ready" ? "severity-info"
    : readiness.overall === "needs_work" ? "severity-warn"
    : "severity-error";

  const overallIcon = readiness.overall === "ready" ? "✓"
    : readiness.overall === "almost_ready" ? "●"
    : readiness.overall === "needs_work" ? "⚠"
    : "○";

  return `
    <details class="cloop-panel cloop-panel-wide" open>
      <summary class="cloop-panel-header">
        <span class="panel-icon">◈</span>
        <span class="panel-title">Creator Readiness</span>
        <span class="cloop-readiness-badge ${overallClass}">${overallIcon} ${escapeHtml(readiness.overall.replace("_", " "))}</span>
        <span class="section-meta">${readiness.packageCount} pkg · ${readiness.capabilityCount} cap · ${readiness.surfaceCount} surf</span>
      </summary>
      <div class="cloop-panel-body">
        <div class="cloop-metrics">
          <div class="cloop-metric">
            <strong>${readiness.packageCount}</strong>
            <span class="run-meta-item">packages</span>
          </div>
          <div class="cloop-metric">
            <strong>${readiness.capabilityCount}</strong>
            <span class="run-meta-item">capabilities</span>
          </div>
          <div class="cloop-metric">
            <strong>${readiness.surfaceCount}</strong>
            <span class="run-meta-item">surfaces</span>
          </div>
          <div class="cloop-metric">
            <strong>${readiness.experienceEntryCount}</strong>
            <span class="run-meta-item">experience entries</span>
          </div>
          <div class="cloop-metric">
            <span class="run-status-dot ${readiness.sessionActive ? "status-ok" : "status-info"}"></span>
            <strong>${readiness.sessionActive ? "active" : "inactive"}</strong>
            <span class="run-meta-item">session</span>
          </div>
        </div>

        ${readiness.missingPieces.length > 0 ? `
          <div class="cloop-missing-section">
            <h3 class="slot-title">Missing Pieces</h3>
            <div class="cloop-issue-list">
              ${readiness.missingPieces.map((m) => `
                <div class="cloop-checklist-item severity-warn">
                  <span class="cloop-checklist-icon">⚠</span>
                  <span>${escapeHtml(m)}</span>
                </div>
              `).join("")}
            </div>
          </div>
        ` : `
          <div class="cloop-checklist-item severity-ok">
            <span class="cloop-checklist-icon">✓</span>
            <span>All required pieces are present.</span>
          </div>
        `}

        ${readiness.recommendations.length > 0 ? `
          <div class="cloop-recommendations">
            <h3 class="slot-title">Recommendations</h3>
            <ul class="cloop-rec-list">
              ${readiness.recommendations.map((r) => `<li class="run-meta-item">${escapeHtml(r)}</li>`).join("")}
            </ul>
          </div>
        ` : ""}

        <details class="protocol-preview-details">
          <summary class="protocol-preview-summary">Public protocol shape (creator readiness)</summary>
          <pre class="protocol-preview-code">${formatJson({
            readiness: {
              overall: "ready|almost_ready|needs_work|unknown",
              package_count: 0,
              capability_count: 0,
              surface_count: 0,
              experience_entry_count: 0,
              session_active: false,
              missing_pieces: ["...", "..."],
              recommendations: ["...", "..."],
            },
          })}</pre>
        </details>
      </div>
    </details>
  `;
}

// ========================================================================
// Panel: Template to Playable
// ========================================================================

function renderTemplateToPlayablePanel(
  steps: WalkthroughStep[],
  recommendations: TemplateRecommendation[],
): string {
  const completedSteps = steps.filter((s) => s.status === "complete").length;
  const totalSteps = steps.length;

  return `
    <details class="cloop-panel cloop-panel-wide" open>
      <summary class="cloop-panel-header">
        <span class="panel-icon">◈</span>
        <span class="panel-title">Template to Playable</span>
        <span class="section-meta">${completedSteps}/${totalSteps} steps complete</span>
      </summary>
      <div class="cloop-panel-body">
        <p class="workspace-note">
          Walk through the steps to go from a new template package to a playable experience.
          Each step checks what's already in place and shows what's needed next.
          Third-party packages drive these panels — no official package hardcoding.
        </p>

        <!-- Walkthrough steps -->
        <div class="cloop-walkthrough">
          <h3 class="slot-title">Walkthrough</h3>
          <div class="cloop-step-list">
            ${steps.map(renderWalkthroughStep).join("")}
          </div>
        </div>

        <!-- Template recommendations -->
        ${recommendations.length > 0 ? `
          <div class="cloop-template-section">
            <h3 class="slot-title">Recommended Templates</h3>
            <p class="run-meta-item" style="margin: 0 0 0.5rem;">
              Packages not yet detected in your workspace. These templates match what's missing:
            </p>
            <div class="cloop-template-grid">
              ${recommendations.map(renderTemplateRecommendation).join("")}
            </div>
          </div>
        ` : `
          <div class="cloop-checklist-item severity-ok" style="margin-top: 0.8rem;">
            <span class="cloop-checklist-icon">✓</span>
            <span>All template types accounted for. Existing packages cover the major surface slots.</span>
          </div>
        `}

        <details class="protocol-preview-details">
          <summary class="protocol-preview-summary">Public protocol shape (walkthrough step)</summary>
          <pre class="protocol-preview-code">${formatJson({
            walkthrough: {
              steps: [
                { order: 1, title: "Initialize Package", description: "...", status: "complete|in_progress|pending|skipped" },
                { order: 2, title: "Declare Capabilities", description: "...", status: "complete|in_progress|pending|skipped" },
              ],
              template_recommendations: [
                { id: "experience", label: "Experience Package", template_type: "experience", suggested_for: "..." },
              ],
            },
          })}</pre>
        </details>
      </div>
    </details>
  `;
}

function renderWalkthroughStep(step: WalkthroughStep): string {
  const statusClass = step.status === "complete" ? "severity-ok"
    : step.status === "in_progress" ? "severity-info"
    : step.status === "pending" ? ""
    : "severity-warn";

  const statusIcon = step.status === "complete" ? "✓"
    : step.status === "in_progress" ? "●"
    : step.status === "skipped" ? "○"
    : "○";

  return `
    <div class="cloop-step-entry ${statusClass ? `status-${step.status}` : ""}">
      <div class="cloop-step-header">
        <span class="cloop-step-number">${step.order}</span>
        <span class="cloop-step-icon ${statusClass}">${statusIcon}</span>
        <strong class="cloop-step-title">${escapeHtml(step.title)}</strong>
        <span class="cloop-step-badge ${statusClass}">${escapeHtml(step.status.replace("_", " "))}</span>
      </div>
      <p class="cloop-step-desc">${escapeHtml(step.description)}</p>
      ${step.actionCommand ? `
        <div class="cloop-step-command">
          <code>${escapeHtml(step.actionCommand)}</code>
        </div>
      ` : ""}
    </div>
  `;
}

function renderTemplateRecommendation(rec: TemplateRecommendation): string {
  return `
    <div class="cloop-template-card">
      <div class="cloop-template-header">
        <strong>${escapeHtml(rec.label)}</strong>
        <span class="surface-chip">${escapeHtml(rec.templateType)}</span>
      </div>
      <p class="cloop-template-desc">${escapeHtml(rec.description)}</p>
      <p class="run-meta-item"><strong>Suggested for:</strong> ${escapeHtml(rec.suggestedFor)}</p>
      ${rec.commands.length > 0 ? `
        <div class="cloop-template-commands">
          ${rec.commands.map((cmd) => `<code>${escapeHtml(cmd)}</code>`).join("")}
        </div>
      ` : ""}
      <details class="protocol-preview-details">
        <summary class="protocol-preview-summary">Prerequisites</summary>
        <ul style="margin: 0.3rem 0; padding-left: 1.2rem;">
          ${rec.prerequisites.map((p) => `<li class="run-meta-item">${escapeHtml(p)}</li>`).join("")}
        </ul>
      </details>
    </div>
  `;
}

// ========================================================================
// Panel: Package Diagnostics Explainability
// ========================================================================

function renderPackageDiagnosticsPanel(diagnostics: PackageDiagnosticCard[]): string {
  const totalIssues = diagnostics.reduce((sum, d) => sum + d.issueCount, 0);
  const totalStrengths = diagnostics.reduce((sum, d) => sum + d.strengthCount, 0);

  return `
    <details class="cloop-panel" open>
      <summary class="cloop-panel-header">
        <span class="panel-icon">◈</span>
        <span class="panel-title">Package Diagnostics Explainability</span>
        <span class="section-meta">${diagnostics.length} pkg · ${totalIssues} issue${totalIssues === 1 ? "" : "s"} · ${totalStrengths} strength${totalStrengths === 1 ? "" : "s"}</span>
      </summary>
      <div class="cloop-panel-body">
        ${diagnostics.length === 0 ? `
          <p class="empty">No packages loaded. Diagnostics appear once packages are registered.</p>
        ` : `
          <div class="cloop-diagnostics-list">
            ${diagnostics.map(renderPackageDiagnosticCard).join("")}
          </div>
        `}

        <details class="protocol-preview-details">
          <summary class="protocol-preview-summary">Public protocol shape (package diagnostic)</summary>
          <pre class="protocol-preview-code">${formatJson({
            package_id: "<package-id>",
            state: "loaded|activated|error",
            entry_kind: "subprocess|builtin",
            issues: [{ severity: "error|warn|info", message: "...", code: "missing_capability" }],
            strengths: ["Capabilities registered", "Surfaces contributed"],
            summary: "Package is operational with N capabilities and M surfaces.",
          })}</pre>
        </details>
      </div>
    </details>
  `;
}

function renderPackageDiagnosticCard(diag: PackageDiagnosticCard): string {
  const stateClass = diag.state === "activated" ? "status-ok"
    : diag.state === "error" ? "status-error"
    : "status-info";

  return `
    <div class="cloop-diag-entry">
      <div class="cloop-diag-header">
        <span class="run-status-dot ${stateClass}"></span>
        <strong>${escapeHtml(diag.packageId)}</strong>
        <span class="surface-chip ${stateClass}">${escapeHtml(diag.state)}</span>
        <span class="surface-chip">${escapeHtml(diag.entryKind)}</span>
      </div>

      <p class="cloop-diag-summary">${escapeHtml(diag.summary)}</p>

      ${diag.issues.length > 0 ? `
        <div class="cloop-diag-section">
          <span class="run-meta-label">Issues (${diag.issueCount}):</span>
          <div class="cloop-issue-list">
            ${diag.issues.map((issue) => `
              <div class="cloop-checklist-item severity-${issue.severity}">
                <span class="cloop-checklist-icon">${issue.severity === "error" ? "⊘" : issue.severity === "warn" ? "⚠" : "◈"}</span>
                <span>${escapeHtml(issue.message)}</span>
                ${issue.code ? `<span class="surface-chip">${escapeHtml(issue.code)}</span>` : ""}
              </div>
            `).join("")}
          </div>
        </div>
      ` : ""}

      ${diag.strengths.length > 0 ? `
        <div class="cloop-diag-section">
          <span class="run-meta-label">Strengths (${diag.strengthCount}):</span>
          <div class="cloop-strength-list">
            ${diag.strengths.map((s) => `
              <div class="cloop-checklist-item severity-ok">
                <span class="cloop-checklist-icon">✓</span>
                <span>${escapeHtml(s)}</span>
              </div>
            `).join("")}
          </div>
        </div>
      ` : ""}
    </div>
  `;
}

// ========================================================================
// Panel: Composition Readiness
// ========================================================================

function renderCompositionReadinessPanel(compositions: CompositionReadiness[]): string {
  const validCount = compositions.filter((c) => c.status === "valid").length;
  const issuesCount = compositions.filter((c) => c.status !== "valid").length;

  return `
    <details class="cloop-panel" open>
      <summary class="cloop-panel-header">
        <span class="panel-icon">◈</span>
        <span class="panel-title">Composition Readiness</span>
        <span class="section-meta">${compositions.length} composition${compositions.length === 1 ? "" : "s"} · ${validCount} valid · ${issuesCount} need${issuesCount === 1 ? "s" : ""} work</span>
      </summary>
      <div class="cloop-panel-body">
        ${compositions.length === 0 ? `
          <p class="empty">No compositions to evaluate. Load packages to assess composition readiness.</p>
        ` : `
          <div class="cloop-composition-list">
            ${compositions.map(renderCompositionEntry).join("")}
          </div>
        `}

        <details class="protocol-preview-details">
          <summary class="protocol-preview-summary">Public protocol shape (composition readiness)</summary>
          <pre class="protocol-preview-code">${formatJson({
            composition: {
              id: "<composition-id>",
              status: "valid|incomplete|invalid|unknown",
              packages: ["<package-id>"],
              missing_packages: ["<package-id>"],
              surface_slots: ["experience_entry", "forge_panel"],
              uncovered_slots: ["experience_entry"],
              check_result: "All required slots and capabilities are present.",
            },
          })}</pre>
        </details>
      </div>
    </details>
  `;
}

function renderCompositionEntry(comp: CompositionReadiness): string {
  const statusClass = comp.status === "valid" ? "severity-ok"
    : comp.status === "incomplete" ? "severity-warn"
    : comp.status === "invalid" ? "severity-error"
    : "severity-info";

  return `
    <div class="cloop-composition-entry">
      <div class="cloop-composition-header">
        <span class="safety-badge ${statusClass}">${escapeHtml(comp.status)}</span>
        <strong>${escapeHtml(comp.compositionId)}</strong>
        <span class="run-meta-item">${comp.packageIds.length} package${comp.packageIds.length === 1 ? "" : "s"} · ${comp.surfaceSlots.length} slot${comp.surfaceSlots.length === 1 ? "" : "s"}</span>
      </div>

      <div class="cloop-composition-detail">
        <div class="cloop-composition-meta">
          <span class="run-meta-label">Packages:</span>
          ${comp.packageIds.map((pid) => `<span class="surface-chip">${escapeHtml(pid)}</span>`).join("")}
        </div>

        ${comp.missingPackages.length > 0 ? `
          <div class="cloop-composition-meta">
            <span class="run-meta-label" style="color: #ffe8a0;">Missing packages:</span>
            ${comp.missingPackages.map((pid) => `<span class="surface-chip" style="border: 1px solid rgba(255,220,100,0.3);">${escapeHtml(pid)}</span>`).join("")}
          </div>
        ` : ""}

        <div class="cloop-composition-meta">
          <span class="run-meta-label">Surface slots:</span>
          ${comp.surfaceSlots.map((s) => `<span class="surface-chip">${escapeHtml(s)}</span>`).join("")}
        </div>

        ${comp.uncoveredSlots.length > 0 ? `
          <div class="cloop-composition-meta">
            <span class="run-meta-label" style="color: #ffe8a0;">Uncovered slots:</span>
            ${comp.uncoveredSlots.map((s) => `<span class="surface-chip" style="border: 1px solid rgba(255,220,100,0.3);">${escapeHtml(s)}</span>`).join("")}
          </div>
        ` : ""}

        <div class="cloop-composition-result">
          <span class="run-meta-label">Check result:</span>
          <code>${escapeHtml(comp.checkResult ?? "No result")}</code>
        </div>
      </div>
    </div>
  `;
}

// ========================================================================
// Panel: Fixture / Reload Controls
// ========================================================================

function renderFixtureControlsPanel(controls: FixtureControl[]): string {
  const live = controls.filter((c) => !c.disabled);
  const disabled = controls.filter((c) => c.disabled);

  return `
    <details class="cloop-panel" open>
      <summary class="cloop-panel-header">
        <span class="panel-icon">◈</span>
        <span class="panel-title">Fixture / Reload Controls</span>
        <span class="section-meta">${live.length} live · ${disabled.length} disabled-safe</span>
      </summary>
      <div class="cloop-panel-body">
        <p class="workspace-note">
          These controls show public-protocol payload previews only — no actual runtime invocation.
          The expected payload shapes document what a third-party fixture/lab package would consume.
        </p>

        ${live.length > 0 ? `
          <div class="cloop-fixture-section">
            <h3 class="slot-title">Live controls</h3>
            <div class="cloop-fixture-list">
              ${live.map(renderFixtureControl).join("")}
            </div>
          </div>
        ` : `
          <p class="empty">No packages loaded — no live fixture controls available.</p>
        `}

        ${disabled.length > 0 ? `
          <div class="cloop-fixture-section" style="margin-top: 0.8rem;">
            <h3 class="slot-title">Disabled-safe affordances</h3>
            <div class="cloop-fixture-list">
              ${disabled.map(renderFixtureControl).join("")}
            </div>
          </div>
        ` : ""}
      </div>
    </details>
  `;
}

function renderFixtureControl(control: FixtureControl): string {
  const actionIcon = control.action === "check" ? "◇"
    : control.action === "conformance" ? "◎"
    : control.action === "run-fixture" ? "▶"
    : "↻";

  return `
    <div class="cloop-fixture-entry ${control.disabled ? "cloop-disabled-safe" : ""}">
      <div class="cloop-fixture-header">
        <span class="cloop-fixture-icon">${actionIcon}</span>
        <strong>${escapeHtml(control.label)}</strong>
        <span class="surface-chip">${escapeHtml(control.action)}</span>
        ${control.disabled ? `<span class="safety-badge severity-info">disabled-safe</span>` : `<span class="safety-badge severity-ok">ready</span>`}
      </div>
      <p class="cloop-fixture-desc">${escapeHtml(control.description)}</p>
      ${control.disabledReason ? `<p class="cloop-fixture-reason">${escapeHtml(control.disabledReason)}</p>` : ""}
      <details class="protocol-preview-details">
        <summary class="protocol-preview-summary">Payload preview (${escapeHtml(control.action)})</summary>
        <pre class="protocol-preview-code">${escapeHtml(control.payloadPreview)}</pre>
      </details>
      <div class="cloop-fixture-action">
        <button type="button" class="${control.disabled ? "button-disabled-safe" : "button-control"}" ${control.disabled ? "disabled" : ""} title="${escapeHtml(control.disabledReason || `Preview payload for ${control.action}`)}">
          ${escapeHtml(control.action)} ${escapeHtml(control.packageId === "__stub__" ? "" : control.packageId)}
        </button>
      </div>
    </div>
  `;
}

// ========================================================================
// Panel: Replacement & Permission Checklist
// ========================================================================

function renderReplacementChecklistPanel(items: ReplacementChecklistItem[]): string {
  const okCount = items.filter((i) => i.status === "ok").length;
  const warnCount = items.filter((i) => i.status === "warn").length;
  const errorCount = items.filter((i) => i.status === "error").length;
  const infoCount = items.filter((i) => i.status === "info").length;

  // Group by category
  const byCategory = new Map<string, ReplacementChecklistItem[]>();
  for (const item of items) {
    const list = byCategory.get(item.category) ?? [];
    list.push(item);
    byCategory.set(item.category, list);
  }

  return `
    <details class="cloop-panel cloop-panel-wide" open>
      <summary class="cloop-panel-header">
        <span class="panel-icon">◈</span>
        <span class="panel-title">Replacement &amp; Permission Checklist</span>
        <span class="section-meta">${items.length} check${items.length === 1 ? "" : "s"} · ${okCount} ok · ${warnCount} warn · ${errorCount} error · ${infoCount} info</span>
      </summary>
      <div class="cloop-panel-body">
        <p class="workspace-note">
          This checklist summarizes the health and coverage of your workspace. Use it to identify
          what needs attention before publishing or sharing an experience. Each item links to the
          relevant panel above for deeper diagnostics.
        </p>

        ${items.length === 0 ? `
          <p class="empty">No checklist items generated. Load packages to populate the checklist.</p>
        ` : `
          <div class="cloop-checklist-categories">
            ${Array.from(byCategory.entries()).map(([category, categoryItems]) => `
              <div class="cloop-checklist-category">
                <h3 class="slot-title">${escapeHtml(category)}</h3>
                <div class="cloop-checklist-items">
                  ${categoryItems.map(renderChecklistItem).join("")}
                </div>
              </div>
            `).join("")}
          </div>
        `}

        <details class="protocol-preview-details">
          <summary class="protocol-preview-summary">Public protocol shape (checklist item)</summary>
          <pre class="protocol-preview-code">${formatJson({
            checklist: [
              {
                category: "Package Health",
                label: "All packages loaded without errors",
                status: "ok|warn|error|info",
                detail: "All packages are in a healthy state.",
                action_needed: "Optional remediation guidance.",
              },
            ],
          })}</pre>
        </details>
      </div>
    </details>
  `;
}

function renderChecklistItem(item: ReplacementChecklistItem): string {
  const icon = item.status === "ok" ? "✓"
    : item.status === "warn" ? "⚠"
    : item.status === "error" ? "⊘"
    : "◈";

  return `
    <div class="cloop-checklist-item severity-${item.status}">
      <div class="cloop-checklist-header">
        <span class="cloop-checklist-icon">${icon}</span>
        <strong>${escapeHtml(item.label)}</strong>
        <span class="safety-badge severity-${item.status}">${escapeHtml(item.status)}</span>
      </div>
      <p class="cloop-checklist-detail">${escapeHtml(item.detail)}</p>
      ${item.actionNeeded ? `
        <p class="cloop-checklist-action">
          <span class="run-meta-label">Action needed:</span>
          ${escapeHtml(item.actionNeeded)}
        </p>
      ` : ""}
    </div>
  `;
}
