/* ============================================================================
 * Muffin Docs — theme.js (MAX)
 * Path: /docs/assets/js/theme.js
 *
 * Features:
 * - Theme modes: "light" | "dark" | "system"
 * - Persists preference in localStorage
 * - Applies theme via data-theme on <html>
 * - Updates <meta name="color-scheme"> and optional theme-color
 * - Syncs across tabs (storage event)
 * - Reacts to OS changes when in "system"
 * - Optional toggle buttons:
 *     [data-theme-toggle] cycles system -> light -> dark
 *     [data-theme-set="light|dark|system"] sets explicitly
 * - Optional status elements:
 *     [data-theme-label] shows current mode
 *
 * No dependencies.
 * ========================================================================== */

(function () {
  "use strict";

  const CFG = {
    storageKey: "muffin.docs.theme",    // stores "light"|"dark"|"system"
    attrName: "data-theme",             // set on <html>
    defaultMode: "system",              // initial mode if none stored
    enableSystemSync: true,             // listen to prefers-color-scheme changes
    applyColorSchemeMeta: true,         // set <meta name="color-scheme">
    applyThemeColorMeta: true,          // set <meta name="theme-color">
    themeColorLight: "#ffffff",
    themeColorDark: "#0b0d10",
  };

  const MODES = ["system", "light", "dark"];

  function $(sel, root) {
    return (root || document).querySelector(sel);
  }

  function $all(sel, root) {
    return Array.from((root || document).querySelectorAll(sel));
  }

  function isValidMode(m) {
    return MODES.includes(m);
  }

  function getSystemMode() {
    return window.matchMedia && window.matchMedia("(prefers-color-scheme: dark)").matches
      ? "dark"
      : "light";
  }

  function readStoredMode() {
    try {
      const v = localStorage.getItem(CFG.storageKey);
      if (isValidMode(v)) return v;
    } catch (_) {}
    return null;
  }

  function storeMode(mode) {
    try {
      localStorage.setItem(CFG.storageKey, mode);
    } catch (_) {}
  }

  function ensureMeta(name) {
    let m = document.querySelector(`meta[name="${name}"]`);
    if (!m) {
      m = document.createElement("meta");
      m.setAttribute("name", name);
      document.head.appendChild(m);
    }
    return m;
  }

  function setMetaColorScheme(effectiveMode) {
    if (!CFG.applyColorSchemeMeta) return;
    const m = ensureMeta("color-scheme");
    // Advertise both, but order matters for some engines
    m.setAttribute("content", effectiveMode === "dark" ? "dark light" : "light dark");
  }

  function setMetaThemeColor(effectiveMode) {
    if (!CFG.applyThemeColorMeta) return;
    const m = ensureMeta("theme-color");
    m.setAttribute("content", effectiveMode === "dark" ? CFG.themeColorDark : CFG.themeColorLight);
  }

  function applyTheme(mode) {
    const root = document.documentElement;

    const effective = mode === "system" ? getSystemMode() : mode;

    // Store requested mode, but apply effective theme
    root.setAttribute(CFG.attrName, effective);
    root.setAttribute("data-theme-mode", mode); // keep original intent visible to CSS/JS if needed

    setMetaColorScheme(effective);
    setMetaThemeColor(effective);

    // Update labels
    for (const el of $all("[data-theme-label]")) {
      el.textContent = mode === "system" ? `System (${effective})` : mode;
    }

    // Update pressed state for explicit buttons
    for (const b of $all("[data-theme-set]")) {
      const v = (b.getAttribute("data-theme-set") || "").toLowerCase();
      b.setAttribute("aria-pressed", v === mode ? "true" : "false");
    }
  }

  function getCurrentMode() {
    const stored = readStoredMode();
    return stored || CFG.defaultMode;
  }

  function setMode(mode) {
    if (!isValidMode(mode)) mode = CFG.defaultMode;
    storeMode(mode);
    applyTheme(mode);
  }

  function cycleMode() {
    const current = getCurrentMode();
    const i = MODES.indexOf(current);
    const next = MODES[(i + 1) % MODES.length];
    setMode(next);
  }

  function bindUI() {
    // Cycle button
    const toggles = $all("[data-theme-toggle]");
    for (const t of toggles) {
      t.addEventListener("click", cycleMode);
    }

    // Explicit set buttons
    const sets = $all("[data-theme-set]");
    for (const b of sets) {
      b.addEventListener("click", () => {
        const v = (b.getAttribute("data-theme-set") || "").toLowerCase();
        setMode(v);
      });
    }
  }

  function bindCrossTabSync() {
    window.addEventListener("storage", (ev) => {
      if (ev.key !== CFG.storageKey) return;
      const v = ev.newValue;
      if (isValidMode(v)) applyTheme(v);
    });
  }

  function bindSystemSync() {
    if (!CFG.enableSystemSync) return;
    if (!window.matchMedia) return;

    const mql = window.matchMedia("(prefers-color-scheme: dark)");

    function onChange() {
      const mode = getCurrentMode();
      if (mode === "system") applyTheme("system");
    }

    // Support older Safari
    if (typeof mql.addEventListener === "function") {
      mql.addEventListener("change", onChange);
    } else if (typeof mql.addListener === "function") {
      mql.addListener(onChange);
    }
  }

  function boot() {
    // Apply as early as possible to reduce flash
    setMode(getCurrentMode());
    bindUI();
    bindCrossTabSync();
    bindSystemSync();
  }

  boot();
})();
