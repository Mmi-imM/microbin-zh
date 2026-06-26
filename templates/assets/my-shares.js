(function (root, factory) {
  if (typeof module === "object" && module.exports) {
    module.exports = factory();
  } else {
    root.MicrobinMyShares = factory();
  }
})(typeof window !== "undefined" ? window : globalThis, function () {
  const STORAGE_KEY = "microbin_my_shares_v1";
  const STATUS_WORDS = new Set(["success", "incorrect"]);
  const PRIVACY_LABELS = {
    public: "公开",
    unlisted: "仅链接可见",
    readonly: "只读",
    public_password: "公开但需要密码",
    private: "私密",
    secret: "本地加密",
  };

  function storage() {
    if (typeof localStorage === "undefined") {
      return null;
    }
    return localStorage;
  }

  function readShares() {
    const store = storage();
    if (!store) {
      return [];
    }

    try {
      const parsed = JSON.parse(store.getItem(STORAGE_KEY) || "[]");
      return Array.isArray(parsed) ? parsed : [];
    } catch (_error) {
      return [];
    }
  }

  function visibleShares() {
    return readShares().filter((share) => !share.gone);
  }

  function writeShares(shares) {
    const store = storage();
    if (!store) {
      return;
    }
    store.setItem(STORAGE_KEY, JSON.stringify(shares));
  }

  function extractId(url) {
    if (!url) {
      return "";
    }

    let parsed;
    try {
      parsed = new URL(url, typeof window !== "undefined" ? window.location.origin : "http://localhost");
    } catch (_error) {
      return "";
    }

    const parts = parsed.pathname.split("/").filter(Boolean);
    while (parts.length && STATUS_WORDS.has(parts[parts.length - 1])) {
      parts.pop();
    }

    if (!parts.length) {
      return "";
    }

    const knownPrefixes = new Set([
      "auth",
      "auth_url",
      "auth_raw",
      "auth_file",
      "auth_edit_private",
      "auth_remove_private",
      "edit",
      "file",
      "p",
      "qr",
      "raw",
      "remove",
      "secure_file",
      "u",
      "upload",
      "url",
    ]);

    if (parts.length >= 2 && knownPrefixes.has(parts[0])) {
      return decodeURIComponent(parts[1]);
    }

    return decodeURIComponent(parts[parts.length - 1]);
  }

  function isGoneStatus(status) {
    return status === 404 || status === 410;
  }

  function isNotFoundHtml(html) {
    return /<h2>\s*404\s*<\/h2>/i.test(html || "") && /Not Found/i.test(html || "");
  }

  function privacyLabel(privacy) {
    return PRIVACY_LABELS[privacy] || privacy || "未知";
  }

  function addShare(input) {
    const id = input.id || extractId(input.url);
    if (!id) {
      return null;
    }

    const now = new Date().toISOString();
    const share = {
      id,
      url: input.url,
      kind: input.kind || "upload",
      privacy: input.privacy || "unlisted",
      privacyLabel: input.privacyLabel || privacyLabel(input.privacy || "unlisted"),
      title: input.title || id,
      createdAt: input.createdAt || now,
      gone: Boolean(input.gone),
    };

    const shares = readShares().filter((item) => item.id !== id);
    shares.unshift(share);
    writeShares(shares.slice(0, 100));
    return share;
  }

  function removeShare(id) {
    const before = readShares();
    const after = before.filter((share) => share.id !== id);
    writeShares(after);
    return before.length - after.length;
  }

  function clearShares() {
    writeShares([]);
  }

  function copyText(text, button) {
    const original = button.textContent;
    const done = () => {
      button.textContent = "已复制";
      setTimeout(() => {
        button.textContent = original;
      }, 1000);
    };

    if (navigator.clipboard && navigator.clipboard.writeText) {
      navigator.clipboard.writeText(text).then(done);
      return;
    }

    const area = document.createElement("textarea");
    area.value = text;
    area.style.position = "fixed";
    area.style.left = "-9999px";
    document.body.appendChild(area);
    area.focus();
    area.select();
    document.execCommand("copy");
    document.body.removeChild(area);
    done();
  }

  function renderShares(container, options) {
    if (!container) {
      return;
    }

    const shares = visibleShares();
    if (!shares.length) {
      container.innerHTML = "";
      return;
    }

    const publicPath = options && options.publicPath ? options.publicPath : "";
    container.innerHTML = `
      <section class="my-shares-panel">
        <div class="my-shares-header">
          <div>
            <h3>我的分享</h3>
            <p>只保存在当前浏览器，不会记录密码。失效链接会自动从这里移除。</p>
          </div>
        </div>
        <div style="width: 100%; overflow-x: auto;">
          <table style="width: 100%; min-width: 620px;">
            <thead>
              <tr>
                <th>分享码</th>
                <th>类型</th>
                <th>创建时间</th>
                <th>操作</th>
              </tr>
            </thead>
            <tbody>
              ${shares
                .map(
                  (share) => `
                    <tr data-share-id="${escapeHtml(share.id)}">
                      <td><a href="${escapeAttr(share.url || `${publicPath}/${share.id}`)}">${escapeHtml(share.title || share.id)}</a></td>
                      <td>${escapeHtml(share.privacyLabel || privacyLabel(share.privacy))}</td>
                      <td>${escapeHtml(formatTime(share.createdAt))}</td>
                      <td>
                        <a href="${escapeAttr(share.url || `${publicPath}/${share.id}`)}">打开</a>
                        <button type="button" class="small-button my-share-copy" data-url="${escapeAttr(share.url || `${publicPath}/${share.id}`)}">复制</button>
                        <button type="button" class="small-button my-share-remove" data-id="${escapeAttr(share.id)}">移除</button>
                      </td>
                    </tr>
                  `
                )
                .join("")}
            </tbody>
          </table>
        </div>
      </section>`;

    container.querySelectorAll(".my-share-copy").forEach((button) => {
      button.addEventListener("click", () => copyText(button.getAttribute("data-url"), button));
    });

    container.querySelectorAll(".my-share-remove").forEach((button) => {
      button.addEventListener("click", () => {
        removeShare(button.getAttribute("data-id"));
        renderShares(container, options);
      });
    });

    checkShares(container, { render: () => renderShares(container, options) });
  }

  function checkShares(container, hooks) {
    if (typeof fetch === "undefined") {
      return Promise.resolve();
    }

    const checks = visibleShares().map((share) => {
      if (!share.url || share.gone) {
        return Promise.resolve();
      }

      const removeGoneShare = () => {
        if (removeShare(share.id) > 0) {
          if (hooks && typeof hooks.render === "function") {
            hooks.render();
          } else if (container) {
            const row = container.querySelector(`[data-share-id="${cssEscape(share.id)}"]`);
            if (row) {
              row.remove();
            }
          }
        }
      };

      return fetch(share.url, {
        method: "HEAD",
        credentials: "same-origin",
        redirect: "manual",
      })
        .then((response) => {
          if (isGoneStatus(response.status)) {
            removeGoneShare();
            return null;
          }

          return fetch(share.url, {
            method: "GET",
            credentials: "same-origin",
          });
        })
        .then((response) => {
          if (!response) {
            return;
          }
          if (isGoneStatus(response.status)) {
            removeGoneShare();
            return;
          }
          if (response.status === 200 && typeof response.text === "function") {
            return response.text().then((html) => {
              if (isNotFoundHtml(html)) {
                removeGoneShare();
              }
            });
          }
        })
        .catch(() => {
          // Network errors should not delete local history.
        });
    });

    return Promise.all(checks);
  }

  function formatTime(value) {
    if (!value) {
      return "";
    }
    const date = new Date(value);
    if (Number.isNaN(date.getTime())) {
      return value;
    }
    return date.toLocaleString("zh-CN", {
      month: "2-digit",
      day: "2-digit",
      hour: "2-digit",
      minute: "2-digit",
    });
  }

  function escapeHtml(value) {
    return String(value || "")
      .replace(/&/g, "&amp;")
      .replace(/</g, "&lt;")
      .replace(/>/g, "&gt;")
      .replace(/"/g, "&quot;")
      .replace(/'/g, "&#39;");
  }

  function escapeAttr(value) {
    return escapeHtml(value).replace(/`/g, "&#96;");
  }

  function cssEscape(value) {
    if (typeof CSS !== "undefined" && CSS.escape) {
      return CSS.escape(value);
    }
    return String(value).replace(/["\\]/g, "\\$&");
  }

  return {
    addShare,
    readShares,
    removeShare,
    clearShares,
    renderShares,
    _private: {
      checkShares,
      extractId,
      isNotFoundHtml,
      isGoneStatus,
      privacyLabel,
      visibleShares,
    },
  };
});
