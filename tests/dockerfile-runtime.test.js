const assert = require("assert");
const fs = require("fs");
const path = require("path");

const dockerfile = fs.readFileSync(
  path.join(__dirname, "../Dockerfile"),
  "utf8"
);

assert(
  dockerfile.includes("FROM debian:trixie-slim"),
  "runtime image should use the same Debian family as the Rust build image"
);
assert(
  dockerfile.includes("xz-utils") && dockerfile.includes("bzip2"),
  "runtime image should include compression libraries used by the release binary"
);
assert(
  dockerfile.includes("useradd --uid 65532"),
  "runtime image should keep running as the nonroot uid"
);

console.log("dockerfile-runtime tests passed");
