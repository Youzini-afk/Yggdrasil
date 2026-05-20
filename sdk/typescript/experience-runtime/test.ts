import { runExperienceRuntimeSelfTest } from "./index.js";

const result = runExperienceRuntimeSelfTest();

for (const r of result.results) {
  if (r.ok) {
    console.log(`  PASS  ${r.label}`);
  } else {
    console.log(`  FAIL  ${r.label}${r.detail ? ` — ${r.detail}` : ""}`);
  }
}

console.log(`\nExperience Runtime SDK self-test: ${result.passed} passed, ${result.failed} failed`);

if (result.failed > 0) {
  process.exit(1);
}
