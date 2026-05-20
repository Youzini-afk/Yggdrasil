/**
 * Standalone test runner for the Agentic Forge SDK.
 *
 * Run via: tsc -p tsconfig.json && node dist/test.js
 */

import { runAgenticForgeSelfTest } from "./index.js";

const result = runAgenticForgeSelfTest();

console.log("=== Agentic Forge SDK Self-Test ===");
console.log(`Passed: ${result.passed}`);
console.log(`Failed: ${result.failed}`);

if (result.failures.length > 0) {
  console.log("\nFailures:");
  for (const f of result.failures) {
    console.log(`  ✗ ${f}`);
  }
  console.log("\n--- TEST FAILED ---");
  process.exit(1);
} else {
  console.log("\n--- ALL TESTS PASSED ---");
  process.exit(0);
}
