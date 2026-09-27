import { createRequire } from "node:module";

const require = createRequire(import.meta.url);

function load() {
  // A createRequire binding is available in ESM even though Node's ambient
  // CommonJS require global is not.
  try {
    // oxlint-disable-next-line scriptc/corpus-no-bare-imports -- deliberate MODULE_NOT_FOUND case
    require("surely-not-installed-anywhere-pr94");
  } catch {
    return "missing";
  }
  return "SHOULD NOT PRINT";
}

console.log(load());
