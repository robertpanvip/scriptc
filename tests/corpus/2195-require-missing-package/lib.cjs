// @ts-check
let status = "unset";
try {
  // oxlint-disable-next-line scriptc/corpus-no-bare-imports -- deliberate MODULE_NOT_FOUND case
  require("scriptc-test-definitely-not-installed");
  status = "installed";
} catch (e) {
  status = "missing: " + String(e.message).split("\n")[0];
}
exports.probe = function probe() { return status; };
