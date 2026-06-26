const assert = require("assert");
const fs = require("fs");
const path = require("path");

const template = fs.readFileSync(
  path.join(__dirname, "../templates/index.html"),
  "utf8"
);

const fileOversizedMatch = template.match(
  /function fileOversized\(\) \{[\s\S]*?\n    \}/
);

assert(fileOversizedMatch, "fileOversized function should exist");

const fileOversized = fileOversizedMatch[0];
const encryptedLimitIndex = fileOversized.indexOf(
  "args.max_file_size_encrypted_mb"
);
const unencryptedLimitIndex = fileOversized.indexOf(
  "args.max_file_size_unencrypted_mb"
);

assert(
  encryptedLimitIndex !== -1 && unencryptedLimitIndex !== -1,
  "fileOversized should check both encrypted and unencrypted limits"
);
assert(
  encryptedLimitIndex < unencryptedLimitIndex,
  "encrypted limit branch should be checked before unencrypted limit branch"
);

const encryptedBranch = fileOversized.slice(0, unencryptedLimitIndex);

assert(
  encryptedBranch.includes('"secret"'),
  "client-side encrypted shares should use encrypted upload limit"
);
assert(
  encryptedBranch.includes('"private"'),
  "private shares should use encrypted upload limit"
);
assert(
  encryptedBranch.includes('"public_password"'),
  "public password shares should use encrypted upload limit"
);

console.log("index-template tests passed");
