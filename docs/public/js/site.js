/* ============================================================================
 * Muffin Docs — site.js (MAX)
 * Path: /docs/assets/js/site.js
 *
 * Responsibilities:
 * - Bootstraps client behaviors (theme init, search init, toc init, copy init)
 * - Mobile sidebar drawer (open/close, focus trap, ESC)
 * - Active nav highlighting (based on location.pathname)
 * - External links: rel="noopener" + target, optional icon
 * - Smooth anchor scrolling with header offset
 * - Adds heading anchor links (h2/h3/h4)
 * - Simple "scroll to top" button (optional)
 *
 * No dependencies.
 * ========================================================================== */

(function () {
  "use strict";

  const CFG = {
    headerSelector: ".site-header",
    sidebarSelector: ".site-sidebar",
    drawerSelector: ".drawer",
    drawerBackdropClass: "drawer-backdrop",
    drawerOpenAttr: "data-drawer-open",

    navLinkSelector: ".nav a[href]",
    tocSelector: ".toc",
    contentSelector: ".site-content",

    mobileBreakpointPx: 920,

    // Anchors
    headingAnchors: ["h2", "h3", "h4"],
    anchorLinkClass: "anchor-link",

    // External links
    externalLinkSelector: "a[href]",
    externalLinkIcon: false, // set true if you want a small icon appended
  };

  /* -------------------------------------------------------------------------
   * Utilities
   * ----------------------------------------------------------------------- */

  function $(sel, root) {
    return (root || document).querySelector(sel);
  }

  function $all(sel, root) {
    return Array.from((root || document).querySelectorAll(sel));
  }

  function clamp(n, a, b) {
    return Math.max(a, Math.min(b, n));
  }

  function isMobile() {
    return window.matchMedia(`(max-width: ${CFG.mobileBreakpointPx}px)`).matches;
  }

  function getHeaderHeight() {
    const h = $(CFG.headerSelector);
    if (!h) return 0;
    return Math.max(0, h.getBoundingClientRect().height || 0);
  }

  function normalizePath(p) {
    if (!p) return "/";
    // remove trailing slash except root
    if (p.length > 1 && p.endsWith("/")) return p.slice(0, -1);
    return p;
  }

  function isExternalHref(href) {
    try {
      const u = new URL(href, window.location.href);
      return u.origin !== window.location.origin;
    } catch {
      return false;
    }
  }

  /* -------------------------------------------------------------------------
   * 1) Active nav highlighting
   * ----------------------------------------------------------------------- */

  function markActiveNav() {
    const path = normalizePath(window.location.pathname || "/");
    const links = $all(CFG.navLinkSelector);

    let best = null;
    let bestLen = -1;

    for (const a of links) {
      const href = a.getAttribute("href") || "";
      if (!href || href.startsWith("#")) continue;

      // Support absolute/relative
      let p = "";
      try {
        const u = new URL(href, window.location.href);
        p = normalizePath(u.pathname);
      } catch {
        p = normalizePath(href);
      }

      // Exact match or longest prefix match
      if (p === path) {
        best = a;
        bestLen = p.length;
        break;
      }
      if (path.startsWith(p) && p.length > bestLen) {
        best = a;
        bestLen = p.length;
      }
    }

    for (const a of links) {
      a.removeAttribute("aria-current");
    }
    if (best) best.setAttribute("aria-current", "page");
  }

  /* -------------------------------------------------------------------------
   * 2) External links hardening
   * ----------------------------------------------------------------------- */

  function hardenExternalLinks() {
    const links = $all(CFG.externalLinkSelector);
    for (const a of links) {
      const href = a.getAttribute("href") || "";
      if (!href) continue;
      if (href.startsWith("#")) continue;
      if (!isExternalHref(href)) continue;

      // Security best practices
      const rel = (a.getAttribute("rel") || "").split(/\s+/).filter(Boolean);
      if (!rel.includes("noopener")) rel.push("noopener");
      if (!rel.includes("noreferrer")) rel.push("noreferrer");
      a.setAttribute("rel", rel.join(" "));

      // Optional target
      if (!a.hasAttribute("target")) a.setAttribute("target", "_blank");

      // Optional icon
      if (CFG.externalLinkIcon && !a.querySelector(":scope > .ext-icon")) {
        const icon = document.createElement("span");
        icon.className = "ext-icon";
        icon.textContent = "↗";
        icon.style.marginLeft = "0.25rem";
        icon.style.opacity = "0.65";
        icon.style.fontSize = "0.95em";
        a.appendChild(icon);
      }
    }
  }

  /* -------------------------------------------------------------------------
   * 3) Anchor links on headings
   * ----------------------------------------------------------------------- */

  function slugify(s) {
    return (s || "")
      .toLowerCase()
      .normalize("NFKD")
      .replace(/[\u0300-\u036f]/g, "")
      .replace(/[^\p{L}\p{N}\s-]+/gu, "")
      .trim()
      .replace(/\s+/g, "-")
      .replace(/-+/g, "-");
  }

  function ensureHeadingIds() {
    for (const tag of CFG.headingAnchors) {
      const nodes = $all(tag);
      for (const h of nodes) {
        if (h.id) continue;
        const id = slugify(h.textContent || "");
        if (!id) continue;
        // avoid duplicates
        let final = id;
        let i = 2;
        while (document.getElementById(final)) {
          final = `${id}-${i++}`;
        }
        h.id = final;
      }
    }
  }

  function addHeadingAnchorLinks() {
    for (const tag of CFG.headingAnchors) {
      const nodes = $all(tag);
      for (const h of nodes) {
        if (!h.id) continue;
        if (h.querySelector(`:scope > a.${CFG.anchorLinkClass}`)) continue;

        const a = document.createElement("a");
        a.className = CFG.anchorLinkClass;
        a.href = `#${h.id}`;
        a.setAttribute("aria-label", "Link to this section");
        a.textContent = "§";

        h.appendChild(a);
      }
    }
  }

  /* -------------------------------------------------------------------------
   * 4) Smooth anchor scrolling with header offset
   * ----------------------------------------------------------------------- */

  function scrollToHash(hash) {
    if (!hash || hash.length < 2) return;
    const id = decodeURIComponent(hash.slice(1));
    const target = document.getElementById(id);
    if (!target) return;

    const y = window.scrollY + target.getBoundingClientRect().top - (getHeaderHeight() + 16);
    window.scrollTo({ top: Math.max(0, y), behavior: "smooth" });
  }

  function enableAnchorOffsetScrolling() {
    // on initial load
    if (window.location.hash) {
      // allow layout to settle
      window.setTimeout(() => scrollToHash(window.location.hash), 50);
    }

    // on hashchange
    window.addEventListener("hashchange", () => {
      scrollToHash(window.location.hash);
    });

    // intercept in-page anchor clicks
    document.addEventListener("click", (ev) => {
      const t = ev.target;
      if (!(t instanceof HTMLElement)) return;

      const a = t.closest('a[href^="#"]');
      if (!a) return;

      const href = a.getAttribute("href") || "";
      if (!href || href === "#") return;

      ev.preventDefault();
      history.pushState(null, "", href);
      scrollToHash(href);
    });
  }

  /* -------------------------------------------------------------------------
   * 5) Mobile drawer (sidebar)
   * ----------------------------------------------------------------------- */

  function createBackdrop() {
    const b = document.createElement("div");
    b.className = CFG.drawerBackdropClass;
    b.setAttribute("data-drawer-backdrop", "1");
    return b;
  }

  function focusTrap(container) {
    const focusable = () =>
      $all(
        'a[href], button:not([disabled]), input:not([disabled]), select:not([disabled]), textarea:not([disabled]), [tabindex]:not([tabindex="-1"])',
        container
      ).filter((el) => el.offsetParent !== null);

    function onKeyDown(ev) {
      if (ev.key !== "Tab") return;

      const els = focusable();
      if (!els.length) return;

      const first = els[0];
      const last = els[els.length - 1];

      if (ev.shiftKey && document.activeElement === first) {
        ev.preventDefault();
        last.focus();
      } else if (!ev.shiftKey && document.activeElement === last) {
        ev.preventDefault();
        first.focus();
      }
    }

    container.addEventListener("keydown", onKeyDown);
    return () => container.removeEventListener("keydown", onKeyDown);
  }

  function openDrawer() {
    if (!isMobile()) return;

    const existing = $(`.${CFG.drawerBackdropClass}`);
    if (existing) return;

    const sidebar = $(CFG.sidebarSelector);
    if (!sidebar) return;

    const backdrop = createBackdrop();
    const drawer = document.createElement("div");
    drawer.className = "drawer";
    drawer.setAttribute("role", "dialog");
    drawer.setAttribute("aria-modal", "true");
    drawer.setAttribute("aria-label", "Navigation");

    // Clone nav to avoid moving DOM
    drawer.appendChild(sidebar.cloneNode(true));

    document.body.appendChild(backdrop);
    document.body.appendChild(drawer);
    document.documentElement.setAttribute(CFG.drawerOpenAttr, "1");

    const removeTrap = focusTrap(drawer);

    function close() {
      removeTrap();
      backdrop.remove();
      drawer.remove();
      document.documentElement.removeAttribute(CFG.drawerOpenAttr);
    }

    backdrop.addEventListener("click", close);
    window.addEventListener(
      "keydown",
      (ev) => {
        if (ev.key === "Escape") close();
      },
      { once: true }
    );

    // Focus first link
    const firstLink = drawer.querySelector("a[href]");
    if (firstLink) firstLink.focus();
  }

  function bindDrawerButtons() {
    // Expect optional buttons:
    // - [data-open-drawer]
    // - [data-close-drawer] (not strictly needed)
    const openBtn = document.querySelector("[data-open-drawer]");
    if (openBtn) openBtn.addEventListener("click", openDrawer);
  }

  /* -------------------------------------------------------------------------
   * 6) Scroll-to-top (optional)
   * ----------------------------------------------------------------------- */

  function ensureScrollTopButton() {
    // Enable if markup exists: <button data-scroll-top class="btn ghost">Top</button>
    const btn = document.querySelector("[data-scroll-top]");
    if (!btn) return;

    function update() {
      const y = window.scrollY || 0;
      btn.style.opacity = y > 700 ? "1" : "0";
      btn.style.pointerEvents = y > 700 ? "auto" : "none";
    }

    btn.addEventListener("click", () => {
      window.scrollTo({ top: 0, behavior: "smooth" });
    });

    update();
    window.addEventListener("scroll", update, { passive: true });
  }

  /* -------------------------------------------------------------------------
   * Boot
   * ----------------------------------------------------------------------- */

  function boot() {
    markActiveNav();
    hardenExternalLinks();

    ensureHeadingIds();
    addHeadingAnchorLinks();
    enableAnchorOffsetScrolling();

    bindDrawerButtons();
    ensureScrollTopButton();
  }

  if (document.readyState === "loading") {
    document.addEventListener("DOMContentLoaded", boot, { once: true });
  } else {
    boot();
  }
})();
