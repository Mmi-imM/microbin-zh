const assert = require("assert");
const fs = require("fs");
const path = require("path");

const source = fs.readFileSync(
  path.join(__dirname, "../src/util/db_sqlite.rs"),
  "utf8"
);

assert(
  source.includes("fn column_exists"),
  "SQLite migration should check whether a column already exists"
);
assert(
  source.includes("fn ensure_column"),
  "SQLite migration should add missing columns through a checked helper"
);
assert(
  !source.includes("let _ = conn.execute(\"ALTER TABLE pasta ADD COLUMN"),
  "SQLite migration should not silently ignore ALTER TABLE errors"
);

console.log("sqlite-migration tests passed");
