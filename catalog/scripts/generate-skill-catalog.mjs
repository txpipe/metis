import { readFileSync, writeFileSync } from "node:fs";
import { dirname, resolve } from "node:path";
import { fileURLToPath } from "node:url";

const scriptDir = dirname(fileURLToPath(import.meta.url));
const catalogDir = resolve(scriptDir, "..");
const manifestPath = resolve(catalogDir, "skill-catalog.manifest.json");
const outputPath = resolve(catalogDir, "skill-catalog.json");

const manifest = JSON.parse(readFileSync(manifestPath, "utf8"));

if (manifest.schemaVersion !== "supernode.skillCatalogManifest/v1") {
  throw new Error(
    `unsupported skill catalog manifest schema: ${manifest.schemaVersion}`,
  );
}

const seen = new Set();
const skills = manifest.skills.map(({ contentPath, ...skill }) => {
  if (!skill.id || seen.has(skill.id)) {
    throw new Error(`invalid or duplicate skill id: ${skill.id}`);
  }
  seen.add(skill.id);

  if (!contentPath) {
    throw new Error(`missing contentPath for skill: ${skill.id}`);
  }

  return {
    ...skill,
    content: readFileSync(resolve(catalogDir, contentPath), "utf8"),
  };
});

const catalog = {
  schemaVersion: "supernode.skillCatalog/v1",
  skills,
};

writeFileSync(outputPath, `${JSON.stringify(catalog, null, 2)}\n`);
