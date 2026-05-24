import ts from "typescript";
import { access, readFile } from "node:fs/promises";
import { fileURLToPath, pathToFileURL } from "node:url";
import { dirname, resolve as pathResolve } from "node:path";

const root = pathResolve(dirname(fileURLToPath(import.meta.url)), "..");

export async function resolve(specifier, context, nextResolve) {
  if (specifier.startsWith("@/")) {
    const resolved = await resolveTsPath(pathResolve(root, "src", specifier.slice(2)));
    return {
      shortCircuit: true,
      url: pathToFileURL(resolved).href,
    };
  }
  if (specifier.startsWith("./") || specifier.startsWith("../")) {
    try {
      return await nextResolve(specifier, context);
    } catch (err) {
      if (err?.code !== "ERR_MODULE_NOT_FOUND") throw err;
      const base = context.parentURL ? dirname(fileURLToPath(context.parentURL)) : root;
      const resolved = await resolveTsPath(pathResolve(base, specifier));
      return {
        shortCircuit: true,
        url: pathToFileURL(resolved).href,
      };
    }
  }
  return nextResolve(specifier, context);
}

async function resolveTsPath(basePath) {
  for (const candidate of [`${basePath}.ts`, `${basePath}.tsx`]) {
    try {
      await access(candidate);
      return candidate;
    } catch {
      // try next extension
    }
  }
  return `${basePath}.ts`;
}

export async function load(url, context, nextLoad) {
  if (!url.endsWith(".ts") && !url.endsWith(".tsx")) {
    return nextLoad(url, context);
  }
  const source = await readFile(fileURLToPath(url), "utf8");
  const output = ts.transpileModule(source, {
    compilerOptions: {
      module: ts.ModuleKind.ES2022,
      target: ts.ScriptTarget.ES2022,
      jsx: ts.JsxEmit.ReactJSX,
      verbatimModuleSyntax: true,
    },
  });
  return { format: "module", shortCircuit: true, source: output.outputText };
}
