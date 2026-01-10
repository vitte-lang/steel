/* ============================================================================
 * Muffin Docs — search.js (MAX)
 * Path: /docs/assets/js/search.js
 *
 * Lightweight client-side search:
 * - Loads a pre-built JSON index (site/data or generated output)
 * - Tokenizes query, performs weighted matching
 * - Ranks results, renders a small dropdown/panel
 * - Keyboard navigation (↑ ↓ Enter Esc)
 * - Works with static hosting (GitHub Pages)
 *
 * Expected HTML (recommended):
 * - An input with: [data-search-input]
 * - A container for results with: [data-search-panel]
 *
 * Minimal markup example:
 *   <input class="input" data-search-input placeholder="Search docs…" />
 *   <div class="search-panel" data-search-panel hidden></div>
 *
 * Index format (recommended):
 * {
 *   "version": 1,
 *   "base_url": "",
 *   "docs": [
 *     { "title": "Install", "path": "/install", "section": "Getting started",
 *       "summary": "Install Muffin", "content": "full text or excerpt", "tags": ["cli"] }
 *   ]
 * }
 *
 * Notes:
 * - For best results, generate "content" as stripped text (no markdown).
 * - This is not a full-text engine; it’s optimized for small/medium doc sets.
 * ========================================================================== */

(function () {
  "use strict";

  /* -------------------------------------------------------------------------
   * Config
   * ----------------------------------------------------------------------- */

  const CFG = {
    indexUrlCandidates: [
      // prefer generated index if present
      "/site/data/search-index.json",
      "/site/generated/search-index.json",
      "/assets/search-index.json",
      "/search-index.json",
    ],

    maxResults: 12,
    minQueryLen: 2,
    debounceMs: 80,

    // Scoring weights
    wTitle: 6.0,
    wSection: 2.2,
    wTags: 2.6,
    wSummary: 2.4,
    wContent: 1.0,
    wPath: 0.6,

    // Boosts
    boostExactTitle: 30,
    boostPrefixTitle: 12,
    boostExactPath: 8,
    boostTagExact: 8,
    boostRecent: 0, // if index provides "updated_at" epoch; keep 0 unless used

    // UI
    highlightClass: "search-hit",
    activeClass: "is-active",
  };

  /* -------------------------------------------------------------------------
   * DOM helpers
   * ----------------------------------------------------------------------- */

  function $(sel, root) {
    return (root || document).querySelector(sel);
  }

  function $all(sel, root) {
    return Array.from((root || document).querySelectorAll(sel));
  }

  function el(tag, attrs) {
    const n = document.createElement(tag);
    if (attrs) {
      for (const [k, v] of Object.entries(attrs)) {
        if (k === "class") n.className = v;
        else if (k === "text") n.textContent = v;
        else if (k === "html") n.innerHTML = v;
        else if (k.startsWith("data-")) n.setAttribute(k, v);
        else n.setAttribute(k, v);
      }
    }
    return n;
  }

  function escapeHtml(s) {
    return (s || "")
      .replace(/&/g, "&amp;")
      .replace(/</g, "&lt;")
      .replace(/>/g, "&gt;")
      .replace(/"/g, "&quot;")
      .replace(/'/g, "&#039;");
  }

  function clamp(n, a, b) {
    return Math.max(a, Math.min(b, n));
  }

  /* -------------------------------------------------------------------------
   * String normalization / tokenization
   * ----------------------------------------------------------------------- */

  function normalize(s) {
    return (s || "")
      .toLowerCase()
      .normalize("NFKD")
      .replace(/[\u0300-\u036f]/g, "") // accents
      .replace(/[_/\\\-]+/g, " ")
      .replace(/[^\p{L}\p{N}\s]+/gu, " ")
      .replace(/\s+/g, " ")
      .trim();
  }

  function tokenize(s) {
    const n = normalize(s);
    if (!n) return [];
    return n.split(" ").filter(Boolean);
  }

  function includesAllTokens(hay, tokens) {
    for (const t of tokens) {
      if (!hay.includes(t)) return false;
    }
    return true;
  }

  function countTokenHits(hay, tokens) {
    let hits = 0;
    for (const t of tokens) {
      if (hay.includes(t)) hits++;
    }
    return hits;
  }

  function startsWithAny(hay, tokens) {
    for (const t of tokens) {
      if (hay.startsWith(t)) return true;
    }
    return false;
  }

  /* -------------------------------------------------------------------------
   * Index loading
   * ----------------------------------------------------------------------- */

  async function fetchJson(url) {
    const r = await fetch(url, { credentials: "omit", cache: "no-cache" });
    if (!r.ok) throw new Error(`HTTP ${r.status}`);
    return r.json();
  }

  async function loadIndex() {
    // Allow explicit override: <meta name="docs:search-index" content="/search-index.json">
    const meta = document.querySelector('meta[name="docs:search-index"]');
    if (meta && meta.content) {
      return { url: meta.content, data: await fetchJson(meta.content) };
    }

    for (const u of CFG.indexUrlCandidates) {
      try {
        const data = await fetchJson(u);
        return { url: u, data };
      } catch (_) {
        /* try next */
      }
    }

    throw new Error("search index not found");
  }

  /* -------------------------------------------------------------------------
   * Scoring
   * ----------------------------------------------------------------------- */

  function scoreDoc(doc, qTokens, qNorm) {
    // Pre-normalize fields
    const title = normalize(doc.title || "");
    const section = normalize(doc.section || "");
    const summary = normalize(doc.summary || "");
    const content = normalize(doc.content || "");
    const path = normalize(doc.path || "");
    const tags = Array.isArray(doc.tags) ? doc.tags.map(normalize).join(" ") : "";

    // If not even one token appears anywhere, early reject
    const hayAll = `${title} ${section} ${summary} ${tags} ${path} ${content}`;
    const hits = countTokenHits(hayAll, qTokens);
    if (hits === 0) return -1;

    // Require all tokens to appear in at least one of the primary fields (title/summary/content/path/tags)
    // Soft constraint: allow missing token if title hit is strong
    const primaryHay = `${title} ${summary} ${tags} ${path} ${content}`;
    const allInPrimary = includesAllTokens(primaryHay, qTokens);

    // Base weighted score
    let s = 0;
    const hitTitle = countTokenHits(title, qTokens);
    const hitSection = countTokenHits(section, qTokens);
    const hitTags = countTokenHits(tags, qTokens);
    const hitSummary = countTokenHits(summary, qTokens);
    const hitPath = countTokenHits(path, qTokens);
    const hitContent = countTokenHits(content, qTokens);

    s += hitTitle * CFG.wTitle;
    s += hitSection * CFG.wSection;
    s += hitTags * CFG.wTags;
    s += hitSummary * CFG.wSummary;
    s += hitPath * CFG.wPath;
    s += hitContent * CFG.wContent;

    // Exact/prefix boosts (title/path)
    if (qNorm && title === qNorm) s += CFG.boostExactTitle;
    if (qNorm && title.startsWith(qNorm)) s += CFG.boostPrefixTitle;

    if (qNorm && path === qNorm) s += CFG.boostExactPath;

    // Tag exact boost (if query is a single token)
    if (qTokens.length === 1) {
      const t = qTokens[0];
      const tagList = Array.isArray(doc.tags) ? doc.tags.map(normalize) : [];
      if (tagList.includes(t)) s += CFG.boostTagExact;
    }

    // Penalize if tokens are scattered and not all present in primary
    if (!allInPrimary) s *= 0.72;

    // Slight boost if all tokens in title or summary
    if (includesAllTokens(title, qTokens)) s *= 1.18;
    else if (includesAllTokens(summary, qTokens)) s *= 1.10;

    // Prefer fewer but stronger matches (reduce noise)
    const tokenCoverage = hits / qTokens.length;
    s *= clamp(tokenCoverage, 0.55, 1.25);

    // Optional recency boost
    if (CFG.boostRecent && doc.updated_at) {
      const ageDays = (Date.now() - Number(doc.updated_at)) / 86400000;
      if (ageDays >= 0 && ageDays <= 30) s += CFG.boostRecent;
    }

    return s;
  }

  function highlightSnippet(text, qTokens, maxLen) {
    const raw = text || "";
    const norm = normalize(raw);
    if (!norm) return "";

    // Find first token occurrence
    let idx = -1;
    let tok = "";
    for (const t of qTokens) {
      const i = norm.indexOf(t);
      if (i !== -1 && (idx === -1 || i < idx)) {
        idx = i;
        tok = t;
      }
    }

    // If not found, return head
    const safeRaw = raw.replace(/\s+/g, " ").trim();
    if (idx === -1) return escapeHtml(safeRaw.slice(0, maxLen || 160));

    // Map normalized index to raw is complex; use a simpler heuristic:
    // find token in raw (case-insensitive)
    const re = new RegExp(tok.replace(/[.*+?^${}()|[\]\\]/g, "\\$&"), "i");
    const m = re.exec(safeRaw);
    let start = 0;

    if (m) {
      start = Math.max(0, m.index - 40);
    }

    const slice = safeRaw.slice(start, start + (maxLen || 180));
    const esc = escapeHtml(slice);

    // Highlight all tokens (best-effort)
    let out = esc;
    for (const t of qTokens) {
      if (!t) continue;
      const r = new RegExp(`(${t.replace(/[.*+?^${}()|[\]\\]/g, "\\$&")})`, "ig");
      out = out.replace(r, `<mark class="${CFG.highlightClass}">$1</mark>`);
    }

    return (start > 0 ? "…" : "") + out + (safeRaw.length > start + (maxLen || 180) ? "…" : "");
  }

  /* -------------------------------------------------------------------------
   * UI rendering
   * ----------------------------------------------------------------------- */

  function ensurePanel(panel) {
    panel.classList.add("search-panel");
    panel.setAttribute("role", "listbox");
    panel.setAttribute("aria-label", "Search results");
    return panel;
  }

  function clearPanel(panel) {
    panel.innerHTML = "";
    panel.hidden = true;
    panel.removeAttribute("data-open");
  }

  function openPanel(panel) {
    panel.hidden = false;
    panel.setAttribute("data-open", "1");
  }

  function renderResults(panel, inputEl, docs, qTokens, qNorm) {
    panel.innerHTML = "";
    if (!docs.length) {
      const empty = el("div", { class: "search-empty", text: "No results" });
      empty.style.padding = "0.75rem 0.85rem";
      empty.style.color = "var(--fg-muted, rgba(0,0,0,0.66))";
      panel.appendChild(empty);
      openPanel(panel);
      return;
    }

    const ul = el("div", { class: "search-results" });
    ul.style.display = "grid";
    ul.style.gap = "0";

    docs.forEach((d, i) => {
      const item = el("a", {
        class: "search-item",
        href: d.path || "#",
      });
      item.setAttribute("role", "option");
      item.setAttribute("tabindex", "-1");
      item.setAttribute("data-idx", String(i));

      item.style.display = "block";
      item.style.padding = "0.7rem 0.85rem";
      item.style.borderTop = i === 0 ? "0" : "1px solid var(--border, rgba(0,0,0,0.12))";
      item.style.textDecoration = "none";

      const title = el("div", { class: "search-title" });
      title.style.fontWeight = "700";
      title.style.color = "var(--fg, rgba(0,0,0,0.90))";
      title.innerHTML = highlightSnippet(d.title || "Untitled", qTokens, 64);

      const meta = el("div", { class: "search-meta" });
      meta.style.marginTop = "0.18rem";
      meta.style.fontSize = "0.85rem";
      meta.style.color = "var(--fg-muted, rgba(0,0,0,0.66))";
      meta.textContent = d.section ? `${d.section} · ${d.path}` : (d.path || "");

      const snippetSrc = d.summary || d.content || "";
      const snippet = el("div", { class: "search-snippet" });
      snippet.style.marginTop = "0.35rem";
      snippet.style.fontSize = "0.90rem";
      snippet.style.color = "var(--fg-muted, rgba(0,0,0,0.66))";
      snippet.innerHTML = highlightSnippet(snippetSrc, qTokens, 170);

      item.appendChild(title);
      item.appendChild(meta);
      item.appendChild(snippet);

      item.addEventListener("mousemove", () => setActive(panel, i));
      ul.appendChild(item);
    });

    panel.appendChild(ul);
    openPanel(panel);

    // default active
    setActive(panel, 0);

    // a11y: link results with input
    inputEl.setAttribute("aria-controls", panel.id || ensurePanelId(panel));
    inputEl.setAttribute("aria-expanded", "true");
  }

  function ensurePanelId(panel) {
    if (panel.id) return panel.id;
    panel.id = "search-panel-" + Math.random().toString(16).slice(2);
    return panel.id;
  }

  function setActive(panel, idx) {
    const items = $all(".search-item", panel);
    if (!items.length) return;

    idx = clamp(idx, 0, items.length - 1);
    items.forEach((it) => it.classList.remove(CFG.activeClass));
    const active = items[idx];
    active.classList.add(CFG.activeClass);

    // Minimal inline styling fallback; prefer CSS if available
    items.forEach((it) => (it.style.background = "transparent"));
    active.style.background = "rgba(110,86,207,0.12)";

    panel.setAttribute("data-active", String(idx));
  }

  function getActiveIdx(panel) {
    const v = panel.getAttribute("data-active");
    const n = v ? Number(v) : 0;
    return Number.isFinite(n) ? n : 0;
  }

  function activateCurrent(panel) {
    const idx = getActiveIdx(panel);
    const items = $all(".search-item", panel);
    if (!items[idx]) return;
    items[idx].click();
  }

  /* -------------------------------------------------------------------------
   * Search controller
   * ----------------------------------------------------------------------- */

  function debounce(fn, ms) {
    let t = 0;
    return function (...args) {
      window.clearTimeout(t);
      t = window.setTimeout(() => fn.apply(this, args), ms);
    };
  }

  async function main() {
    const inputEl = document.querySelector("[data-search-input]");
    const panelEl = document.querySelector("[data-search-panel]");
    if (!inputEl || !panelEl) return;

    ensurePanel(panelEl);

    // Styling fallback for panel (themes can override via CSS)
    panelEl.style.position = "absolute";
    panelEl.style.zIndex = "80";
    panelEl.style.width = "min(92vw, 720px)";
    panelEl.style.maxHeight = "min(60vh, 540px)";
    panelEl.style.overflow = "auto";
    panelEl.style.marginTop = "0.5rem";
    panelEl.style.border = "1px solid var(--border, rgba(0,0,0,0.12))";
    panelEl.style.borderRadius = "16px";
    panelEl.style.background = "var(--bg, #fff)";
    panelEl.style.boxShadow = "var(--shadow-2, 0 18px 50px rgba(0,0,0,0.10))";
    panelEl.style.backdropFilter = "blur(10px)";
    panelEl.style.webkitBackdropFilter = "blur(10px)";

    let index = null;
    try {
      const loaded = await loadIndex();
      index = loaded.data;
    } catch (e) {
      // silent fail: keep UI but no results
      panelEl.innerHTML = "";
      panelEl.hidden = true;
      return;
    }

    const docs = Array.isArray(index.docs) ? index.docs : [];

    function doSearch(q) {
      const qNorm = normalize(q);
      const qTokens = tokenize(q);
      if (qTokens.length === 0 || qNorm.length < CFG.minQueryLen) {
        clearPanel(panelEl);
        inputEl.setAttribute("aria-expanded", "false");
        return;
      }

      const scored = [];
      for (const d of docs) {
        const s = scoreDoc(d, qTokens, qNorm);
        if (s >= 0) scored.push({ d, s });
      }

      scored.sort((a, b) => b.s - a.s);
      const results = scored.slice(0, CFG.maxResults).map((x) => x.d);

      renderResults(panelEl, inputEl, results, qTokens, qNorm);
    }

    const doSearchDebounced = debounce(doSearch, CFG.debounceMs);

    inputEl.setAttribute("autocomplete", "off");
    inputEl.setAttribute("spellcheck", "false");
    inputEl.setAttribute("aria-autocomplete", "list");
    inputEl.setAttribute("aria-expanded", "false");

    inputEl.addEventListener("input", (ev) => {
      doSearchDebounced(ev.target.value || "");
    });

    inputEl.addEventListener("focus", () => {
      const v = inputEl.value || "";
      if (v.trim().length >= CFG.minQueryLen) doSearchDebounced(v);
    });

    // Key navigation
    inputEl.addEventListener("keydown", (ev) => {
      const open = panelEl.getAttribute("data-open") === "1";
      if (!open) {
        if (ev.key === "ArrowDown" && (inputEl.value || "").trim().length >= CFG.minQueryLen) {
          doSearchDebounced(inputEl.value || "");
          ev.preventDefault();
        }
        return;
      }

      if (ev.key === "Escape") {
        clearPanel(panelEl);
        inputEl.setAttribute("aria-expanded", "false");
        ev.preventDefault();
        return;
      }

      if (ev.key === "Enter") {
        activateCurrent(panelEl);
        ev.preventDefault();
        return;
      }

      if (ev.key === "ArrowDown") {
        const i = getActiveIdx(panelEl);
        setActive(panelEl, i + 1);
        ev.preventDefault();
        return;
      }

      if (ev.key === "ArrowUp") {
        const i = getActiveIdx(panelEl);
        setActive(panelEl, i - 1);
        ev.preventDefault();
        return;
      }
    });

    // Close when clicking outside
    document.addEventListener("click", (ev) => {
      const t = ev.target;
      if (!(t instanceof HTMLElement)) return;
      if (t === inputEl) return;
      if (panelEl.contains(t)) return;
      clearPanel(panelEl);
      inputEl.setAttribute("aria-expanded", "false");
    });

    // If panel gains focus (tabbing), allow esc
    panelEl.addEventListener("keydown", (ev) => {
      if (ev.key === "Escape") {
        clearPanel(panelEl);
        inputEl.focus();
        inputEl.setAttribute("aria-expanded", "false");
        ev.preventDefault();
      }
    });
  }

  if (document.readyState === "loading") {
    document.addEventListener("DOMContentLoaded", main, { once: true });
  } else {
    main();
  }
})();
