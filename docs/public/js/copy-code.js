/* ============================================================================
 * Muffin Docs — copy-code.js (MAX+)
 * Path: /docs/assets/js/copy-code.js
 *
 * What it does:
 * - Auto-adds a copy button to code blocks (<pre><code>...</code></pre>)
 * - Copies *rendered* text (innerText) to preserve newlines
 * - Optional behaviors via data-attributes on <pre>:
 *     data-lang="muf"            -> shows language label (CSS handles)
 *     data-filename="build.muf"  -> shows filename chip (optional)
 *     data-no-copy               -> disables enhancement
 *     data-copy-trim-prompts="1" -> removes leading "$ " / "> " prompts
 *     data-copy-trim-empty="1"   -> trims leading/trailing blank lines (default on)
 *     data-copy-mode="code|pre"  -> source node: <code> (default) or <pre>
 *     data-copy-target="#id"     -> copies textContent of an external node
 * - Works with dynamic content (MutationObserver)
 * - Accessible: focus-visible, aria-live, keyboard support
 * - Inline copy: <code data-copy="...">...</code>
 *
 * CSS hooks expected (see /assets/css/code.css):
 * - pre.has-copy, button.code-copy
 *
 * No dependencies.
 * ========================================================================== */

(function () {
  "use strict";

  /* -------------------------------------------------------------------------
   * Config
   * ----------------------------------------------------------------------- */

  const CFG = {
    selectorPre: "pre",
    selectorCodeInPre: "code",
    selectorInlineCopy: "code[data-copy]",
    skipPreClasses: ["no-copy", "mermaid", "diagram"],

    // Button labels
    labelIdle: "Copy",
    labelBusy: "…",
    labelCopied: "Copied",
    labelFailed: "Failed",

    // Timings
    resetMs: 1400,
    busyMinMs: 120, // ensure UX feedback even if copy is instant

    // Default trims
    defaultTrimEmpty: true,

    // Prompt stripping
    promptPrefixes: ["$ ", "> ", "# ", "❯ ", "➜ "],

    // Optional: set to true to log debug info
    debug: false,
  };

  function dbg(...args) {
    if (CFG.debug) console.log("[copy-code]", ...args);
  }

  /* -------------------------------------------------------------------------
   * Feature detection / utilities
   * ----------------------------------------------------------------------- */

  function hasClipboardAPI() {
    return !!(navigator && navigator.clipboard && typeof navigator.clipboard.writeText === "function");
  }

  async function canWriteClipboard() {
    // Best-effort: permissions API might be blocked; treat as "unknown => try"
    try {
      if (!navigator.permissions || !navigator.permissions.query) return true;
      const res = await navigator.permissions.query({ name: "clipboard-write" });
      return res.state === "granted" || res.state === "prompt";
    } catch {
      return true;
    }
  }

  function sleep(ms) {
    return new Promise((r) => setTimeout(r, ms));
  }

  function nowMs() {
    return (typeof performance !== "undefined" && performance.now) ? performance.now() : Date.now();
  }

  function normalizeNewlines(s) {
    return (s || "").replace(/\r\n/g, "\n").replace(/\r/g, "\n");
  }

  function trimOuterBlankLines(s) {
    // remove leading/trailing blank lines but preserve internal spacing
    return normalizeNewlines(s).replace(/^\s*\n+/g, "").replace(/\n+\s*$/g, "");
  }

  function stripPromptLines(s) {
    // For each line: remove known prompt prefixes.
    const lines = normalizeNewlines(s).split("\n");
    const out = lines.map((ln) => {
      for (const p of CFG.promptPrefixes) {
        if (ln.startsWith(p)) return ln.slice(p.length);
      }
      return ln;
    });
    return out.join("\n");
  }

  function safeClosest(el, selector) {
    try {
      return el.closest(selector);
    } catch {
      return null;
    }
  }

  /* -------------------------------------------------------------------------
   * Clipboard copy implementation
   * ----------------------------------------------------------------------- */

  async function copyText(text) {
    const payload = text ?? "";
    // Prefer Clipboard API
    if (hasClipboardAPI()) {
      try {
        const ok = await canWriteClipboard();
        if (!ok) dbg("clipboard-write permission denied");
        await navigator.clipboard.writeText(payload);
        return true;
      } catch (e) {
        dbg("Clipboard API failed; falling back", e);
      }
    }

    // Fallback: execCommand
    try {
      const ta = document.createElement("textarea");
      ta.value = payload;
      ta.setAttribute("readonly", "");
      ta.style.position = "fixed";
      ta.style.top = "-1000px";
      ta.style.left = "-1000px";
      ta.style.opacity = "0";
      document.body.appendChild(ta);

      ta.focus();
      ta.select();
      ta.setSelectionRange(0, ta.value.length);

      const ok = document.execCommand("copy");
      document.body.removeChild(ta);
      return !!ok;
    } catch (e) {
      dbg("execCommand fallback failed", e);
      return false;
    }
  }

  /* -------------------------------------------------------------------------
   * Button + a11y helpers
   * ----------------------------------------------------------------------- */

  function setBtnState(btn, state) {
    // state: "idle" | "busy" | "copied" | "failed"
    btn.setAttribute("data-state", state);

    if (state === "idle") {
      btn.disabled = false;
      btn.setAttribute("aria-busy", "false");
      btn.textContent = CFG.labelIdle;
      return;
    }

    if (state === "busy") {
      btn.disabled = true;
      btn.setAttribute("aria-busy", "true");
      btn.textContent = CFG.labelBusy;
      return;
    }

    if (state === "copied") {
      btn.disabled = false;
      btn.setAttribute("aria-busy", "false");
      btn.textContent = CFG.labelCopied;
      return;
    }

    if (state === "failed") {
      btn.disabled = false;
      btn.setAttribute("aria-busy", "false");
      btn.textContent = CFG.labelFailed;
      return;
    }
  }

  function ensureLiveRegion() {
    // One global aria-live region for announcements (screen readers)
    let lr = document.getElementById("code-copy-live");
    if (lr) return lr;

    lr = document.createElement("div");
    lr.id = "code-copy-live";
    lr.className = "visually-hidden";
    lr.setAttribute("aria-live", "polite");
    lr.setAttribute("aria-atomic", "true");
    document.body.appendChild(lr);
    return lr;
  }

  function announce(msg) {
    const lr = ensureLiveRegion();
    // Clear then set to force announcement
    lr.textContent = "";
    // microtask
    Promise.resolve().then(() => {
      lr.textContent = msg;
    });
  }

  function createButton() {
    const btn = document.createElement("button");
    btn.type = "button";
    btn.className = "code-copy";
    btn.setAttribute("aria-label", "Copy code to clipboard");
    btn.setAttribute("aria-busy", "false");
    btn.setAttribute("data-state", "idle");
    btn.textContent = CFG.labelIdle;
    return btn;
  }

  /* -------------------------------------------------------------------------
   * Text extraction
   * ----------------------------------------------------------------------- */

  function getCopyTargetText(preEl) {
    // 1) Explicit external target
    const externalSel = preEl.getAttribute("data-copy-target");
    if (externalSel) {
      const ext = document.querySelector(externalSel);
      if (ext) return ext.textContent || "";
    }

    // 2) Choose mode
    const mode = (preEl.getAttribute("data-copy-mode") || "code").toLowerCase();
    const trimEmpty = preEl.getAttribute("data-copy-trim-empty");
    const trimEmptyOn = trimEmpty === null ? CFG.defaultTrimEmpty : trimEmpty === "1" || trimEmpty === "true";

    const trimPrompts = preEl.getAttribute("data-copy-trim-prompts");
    const trimPromptsOn = trimPrompts === "1" || trimPrompts === "true";

    let raw = "";

    if (mode === "pre") {
      raw = preEl.innerText || "";
    } else {
      const codeEl = preEl.querySelector(CFG.selectorCodeInPre);
      raw = (codeEl ? codeEl.innerText : preEl.innerText) || "";
    }

    raw = normalizeNewlines(raw);

    if (trimPromptsOn) raw = stripPromptLines(raw);
    if (trimEmptyOn) raw = trimOuterBlankLines(raw);

    return raw;
  }

  /* -------------------------------------------------------------------------
   * Enhancement logic
   * ----------------------------------------------------------------------- */

  function shouldSkip(preEl) {
    if (!preEl) return true;
    if (preEl.hasAttribute("data-no-copy")) return true;

    for (const cls of CFG.skipPreClasses) {
      if (preEl.classList.contains(cls)) return true;
    }

    // Already enhanced
    if (preEl.querySelector(":scope > button.code-copy")) return true;

    // If no meaningful content
    const t = getCopyTargetText(preEl);
    if (!t || !t.trim()) return true;

    return false;
  }

  function markPre(preEl) {
    preEl.classList.add("has-copy");

    // If data-filename present, create a small chip (optional) as pseudo-label.
    // CSS can style .code-filename if desired.
    const filename = preEl.getAttribute("data-filename");
    if (filename && !preEl.querySelector(":scope > .code-filename")) {
      const chip = document.createElement("div");
      chip.className = "code-filename";
      chip.textContent = filename;
      // minimal inline styles fallback (themes can override via CSS)
      chip.style.position = "absolute";
      chip.style.top = "0.55rem";
      chip.style.left = "0.75rem";
      chip.style.fontFamily = "var(--code-font, ui-monospace)";
      chip.style.fontSize = "0.78rem";
      chip.style.color = "var(--code-muted, rgba(0,0,0,0.55))";
      chip.style.padding = "0.15rem 0.45rem";
      chip.style.borderRadius = "999px";
      chip.style.background = "rgba(127,127,127,0.10)";
      chip.style.border = "1px solid rgba(127,127,127,0.18)";
      chip.style.userSelect = "none";

      // Ensure top padding if filename chip exists
      preEl.style.paddingTop = "calc(var(--code-block-padding-y, 0.9rem) + 1.9rem)";
      preEl.insertBefore(chip, preEl.firstChild);
    }
  }

  async function handleCopyClick(btn, preEl) {
    const started = nowMs();
    setBtnState(btn, "busy");

    const text = getCopyTargetText(preEl);

    // Ensure busy shows for at least busyMinMs
    const copyPromise = copyText(text);
    const minDelay = sleep(CFG.busyMinMs);

    const [ok] = await Promise.all([copyPromise, minDelay]).then((arr) => [arr[0]]);

    if (ok) {
      setBtnState(btn, "copied");
      announce("Code copied to clipboard");
    } else {
      setBtnState(btn, "failed");
      announce("Copy failed");
    }

    const elapsed = nowMs() - started;
    dbg("copy elapsed(ms)", Math.round(elapsed), "ok=", ok);

    window.setTimeout(() => setBtnState(btn, "idle"), CFG.resetMs);
  }

  function enhancePre(preEl) {
    if (shouldSkip(preEl)) return;

    markPre(preEl);

    const btn = createButton();
    btn.addEventListener("click", () => handleCopyClick(btn, preEl));

    // Keyboard: allow Enter/Space on focused button is default; also allow "c" when pre focused
    preEl.addEventListener("keydown", (ev) => {
      // If focus is inside pre (e.g., user tabs into it)
      if (!(ev instanceof KeyboardEvent)) return;
      if (ev.defaultPrevented) return;

      // Ctrl/Cmd + Shift + C while focus is in this pre => copy
      const key = (ev.key || "").toLowerCase();
      const meta = ev.metaKey || false;
      const ctrl = ev.ctrlKey || false;
      const shift = ev.shiftKey || false;

      if ((ctrl || meta) && shift && key === "c") {
        ev.preventDefault();
        handleCopyClick(btn, preEl);
      }
    });

    // Insert button as first child (before code)
    preEl.insertBefore(btn, preEl.firstChild);
  }

  function enhanceAll(root) {
    const scope = root || document;
    const pres = scope.querySelectorAll(CFG.selectorPre);
    for (const pre of pres) enhancePre(pre);
  }

  function observe() {
    if (!("MutationObserver" in window)) return;

    const obs = new MutationObserver((mutations) => {
      for (const m of mutations) {
        for (const node of m.addedNodes) {
          if (!(node instanceof HTMLElement)) continue;

          if (node.tagName === "PRE") {
            enhancePre(node);
            continue;
          }

          if (node.querySelector) {
            const pres = node.querySelectorAll("pre");
            for (const pre of pres) enhancePre(pre);
          }
        }
      }
    });

    obs.observe(document.documentElement, { childList: true, subtree: true });
  }

  /* -------------------------------------------------------------------------
   * Inline copy
   * ----------------------------------------------------------------------- */

  function enableInlineCopy() {
    document.addEventListener("click", async (ev) => {
      const t = ev.target;
      if (!(t instanceof HTMLElement)) return;
      if (!t.matches(CFG.selectorInlineCopy)) return;

      const payload = t.getAttribute("data-copy") || t.innerText || "";
      if (!payload) return;

      const ok = await copyText(payload);
      t.setAttribute("data-copied", ok ? "true" : "false");
      announce(ok ? "Copied" : "Copy failed");

      window.setTimeout(() => t.removeAttribute("data-copied"), 900);
    });
  }

  /* -------------------------------------------------------------------------
   * Boot
   * ----------------------------------------------------------------------- */

  function boot() {
    enhanceAll(document);
    observe();
    enableInlineCopy();
  }

  if (document.readyState === "loading") {
    document.addEventListener("DOMContentLoaded", boot, { once: true });
  } else {
    boot();
  }
})();
