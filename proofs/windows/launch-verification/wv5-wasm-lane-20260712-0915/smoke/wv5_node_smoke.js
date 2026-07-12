// WV-5 node smoke — runs the wasm-pack (--target nodejs) build of garnet-wasm
// and asserts the garnet.wasm.run/1 contract with REAL executed output.
// Mirrors garnet-wasm/tests/run_source.rs fixtures exactly.
const assert = require("assert");
const g = require("C:/garnet/garnet-wasm/pkg-node"); // resolves via pkg package.json "main"

const HELLO = `
@caps()
def main() {
  println("Hello from Garnet!")
  0
}
`;

const res = JSON.parse(g.run_source(HELLO));
assert.strictEqual(res.schema, "garnet.wasm.run/1", `schema: ${res.schema}`);
assert.strictEqual(res.exit_class, "ok", `exit_class: ${JSON.stringify(res)}`);
assert.strictEqual(res.stdout, "Hello from Garnet!\n", `stdout: ${JSON.stringify(res.stdout)}`);
console.log("NODE_SMOKE hello:", JSON.stringify({ exit_class: res.exit_class, stdout: res.stdout }));

// Authority fails closed IN REAL WASM: undeclared proc authority must not be Ok.
const AUTH = `
@caps()
def main() {
  proc::run("echo hi")
  0
}
`;
const res2 = JSON.parse(g.run_source(AUTH));
assert.notStrictEqual(res2.exit_class, "ok", `authority must fail closed: ${JSON.stringify(res2)}`);
assert.ok(res2.diagnostic && res2.diagnostic.length > 0, "diagnostic must be present");
console.log("NODE_SMOKE authority-fail-closed:", res2.exit_class);

console.log("NODE_SMOKE: PASS");
