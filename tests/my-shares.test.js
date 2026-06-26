const assert = require("assert");
const path = require("path");

const shares = require(path.join(__dirname, "../templates/assets/my-shares.js"));

const store = {};

global.window = {
  location: {
    origin: "https://share.312996.xyz",
  },
};

global.localStorage = {
  getItem(key) {
    return Object.prototype.hasOwnProperty.call(store, key) ? store[key] : null;
  },
  setItem(key, value) {
    store[key] = String(value);
  },
  removeItem(key) {
    delete store[key];
  },
};

shares.clearShares();

shares.addShare({
  url: "https://share.312996.xyz/auth/custom-code/success",
  privacy: "private",
});

assert.strictEqual(shares.readShares()[0].id, "custom-code");
assert.strictEqual(shares.readShares()[0].privacyLabel, "私密");

shares.addShare({
  url: "https://share.312996.xyz/upload/public-code",
  privacy: "public_password",
  title: "Important note",
});

assert.strictEqual(shares.readShares()[0].id, "public-code");
assert.strictEqual(shares.readShares()[0].privacyLabel, "公开但需要密码");
assert.strictEqual(shares.readShares()[0].title, "Important note");

assert.strictEqual(shares._private.extractId("https://share.312996.xyz/auth/demo/success"), "demo");
assert.strictEqual(shares._private.extractId("https://share.312996.xyz/auth_url/demo/incorrect"), "demo");
assert.strictEqual(shares._private.extractId("https://share.312996.xyz/demo"), "demo");
assert.strictEqual(shares._private.isGoneStatus(404), true);
assert.strictEqual(shares._private.isGoneStatus(410), true);
assert.strictEqual(shares._private.isGoneStatus(401), false);
assert.strictEqual(shares._private.isGoneStatus(403), false);

global.localStorage.setItem(
  "microbin_my_shares_v1",
  JSON.stringify([
    { id: "old-expired", url: "https://share.312996.xyz/old-expired", gone: true },
    { id: "still-visible", url: "https://share.312996.xyz/still-visible" },
  ])
);
assert.deepStrictEqual(
  shares._private.visibleShares().map((share) => share.id),
  ["still-visible"]
);

async function testGoneSharesAreRemovedAfterStatusCheck() {
  shares.clearShares();
  shares.addShare({
    url: "https://share.312996.xyz/upload/expired-code",
    privacy: "public",
  });

  let renderedAgain = false;
  const container = {
    querySelector() {
      return null;
    },
  };

  global.fetch = async () => ({ status: 404 });

  await shares._private.checkShares(container, {
    render() {
      renderedAgain = true;
    },
  });

  assert.strictEqual(shares.readShares().length, 0);
  assert.strictEqual(renderedAgain, true);
}

async function testNotFoundHtmlRemovesShareEvenWithOkStatus() {
  shares.clearShares();
  shares.addShare({
    url: "https://share.312996.xyz/upload/expired-but-200",
    privacy: "public",
  });

  let renderedAgain = false;
  global.fetch = async (_url, options) => {
    if (options.method === "HEAD") {
      return { status: 200 };
    }

    return {
      status: 200,
      text: async () => "<h2>404</h2><b>Not Found</b>",
    };
  };

  await shares._private.checkShares({ querySelector: () => null }, {
    render() {
      renderedAgain = true;
    },
  });

  assert.strictEqual(shares.readShares().length, 0);
  assert.strictEqual(renderedAgain, true);
}

testGoneSharesAreRemovedAfterStatusCheck()
  .then(testNotFoundHtmlRemovesShareEvenWithOkStatus)
  .then(() => {
    console.log("my-shares tests passed");
  })
  .catch((error) => {
    console.error(error);
    process.exit(1);
  });
