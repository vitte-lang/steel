// toc.js
/* ============================================================================
 * Muffin Docs — toc.js (MAX)
 * Path: /docs/assets/js/toc.js
 *
 * Features:
 * - Builds a Table of Contents from headings in the main content
 * - Supports h2/h3/h4 nesting
 * - Adds ids if missing (slugify)
 * - Updates active TOC entry on scroll (IntersectionObserver preferred)
 * - Smooth scroll to headings with header offset
 * - Optional: collapse/expand h3/h4 groups
 *
 * Expected markup:
 * - Container: [data-toc] (or .toc)
 * - Content root: [data-doc-content] (or .site-content)
 *
 * No dependencies.
 * ========================================================================== */

(function () {
  "use strict";

  const CFG = {
    tocContainerSelector: "[data-toc], .toc",
    contentSelector: "[data-doc-content], .site-content",
    headingsSelector: "h2, h3, h4",
    include: { h2: true, h3: true, h4: true },

    minHeadings: 2,

    headerSelector: ".site-header",
    headerOffsetPx: 16,

    activeClass: "is-active",
    tocLinkClass: "toc-link",
    tocItemClass: "toc-item",
    tocListClass: "toc-list",

    // Nesting: h3 under last h2, h4 under last h3
    maxDepth: 3,

    // Smooth scroll
    smoothScroll: true,

    // Observe
    useIntersectionObserver: true,

    // Optional: collapse groups (requires CSS)
    collapsible: false,
  };

  function $(sel, root) {
    return (root || document).querySelector(sel);
  }

  function $all(sel, root) {
    return Array.from((root || document).querySelectorAll(sel));
  }

  function clamp(n, a, b) {
    return Math.max(a, Math.min(b, n));
  }

  function getHeaderHeight() {
    const h = $(CFG.headerSelector);
    if (!h) return 0;
    return Math.max(0, h.getBoundingClientRect().height || 0);
  }

  function normalize(s) {
    return (s || "")
      .toLowerCase()
      .normalize("NFKD")
      .replace(/[\u0300-\u036f]/g, "")
      .replace(/[^\p{L}\p{N}\s-]+/gu, "")
      .trim()
      .replace(/\s+/g, "-")
      .replace(/-+/g, "-");
  }

  function ensureUniqueId(base) {
    let id = base || "section";
    if (!document.getElementById(id)) return id;

    let i = 2;
    while (document.getElementById(`${id}-${i}`)) i++;
    return `${id}-${i}`;
  }

  function ensureHeadingIds(headings) {
    for (const h of headings) {
      if (h.id) continue;
      const base = normalize(h.textContent || "");
      if (!base) continue;
      h.id = ensureUniqueId(base);
    }
  }

  function headingLevel(h) {
    const t = (h.tagName || "").toLowerCase();
    if (t === "h2") return 2;
    if (t === "h3") return 3;
    if (t === "h4") return 4;
    return 99;
  }

  function shouldInclude(h) {
    const t = (h.tagName || "").toLowerCase();
    if (t === "h2") return !!CFG.include.h2;
    if (t === "h3") return !!CFG.include.h3;
    if (t === "h4") return !!CFG.include.h4;
    return false;
  }

  function buildTree(headings) {
    // Produces a nested structure:
    // [{h, children:[{h, children:[...]}]}]
    const root = [];
    let lastH2 = null;
    let lastH3 = null;

    for (const h of headings) {
      if (!shouldInclude(h)) continue;
      const lvl = headingLevel(h);

      if (lvl === 2) {
        const node = { h, children: [] };
        root.push(node);
        lastH2 = node;
        lastH3 = null;
      } else if (lvl === 3) {
        const node = { h, children: [] };
        if (lastH2) lastH2.children.push(node);
        else root.push(node);
        lastH3 = node;
      } else if (lvl === 4) {
        const node = { h, children: [] };
        if (lastH3) lastH3.children.push(node);
        else if (lastH2) lastH2.children.push(node);
        else root.push(node);
      }
    }

    return root;
  }

  function createLink(h, depth) {
    const a = document.createElement("a");
    a.className = CFG.tocLinkClass;
    a.href = `#${encodeURIComponent(h.id)}`;
    a.textContent = (h.textContent || "").trim();
    a.setAttribute("data-depth", String(depth));
    a.setAttribute("data-target-id", h.id);
    a.style.display = "block";
    a.style.padding = depth === 2 ? "0.28rem 0.35rem" : depth === 3 ? "0.22rem 0.35rem 0.22rem 0.85rem" : "0.20rem 0.35rem 0.20rem 1.35rem";
    a.style.borderRadius = "8px";
    a.style.color = "var(--fg-muted, rgba(0,0,0,0.66))";
    a.style.textDecoration = "none";
    return a;
  }

  function createList(items, depth) {
    const ul = document.createElement("div");
    ul.className = CFG.tocListClass;
    ul.style.display = "grid";
    ul.style.gap = "0";

    for (const it of items) {
      const row = document.createElement("div");
      row.className = CFG.tocItemClass;

      const lvl = headingLevel(it.h);
      const link = createLink(it.h, lvl);

      link.addEventListener("click", (ev) => {
        // Smooth scroll with offset
        if (!CFG.smoothScroll) return;
        ev.preventDefault();
        const y = window.scrollY + it.h.getBoundingClientRect().top - (getHeaderHeight() + CFG.headerOffsetPx);
        history.pushState(null, "", `#${encodeURIComponent(it.h.id)}`);
        window.scrollTo({ top: Math.max(0, y), behavior: "smooth" });
      });

      row.appendChild(link);

      if (it.children && it.children.length) {
        row.appendChild(createList(it.children, depth + 1));
      }

      ul.appendChild(row);
    }

    return ul;
  }

  function renderTOC(container, headings) {
    container.innerHTML = "";

    const title = container.querySelector(".toc-title") || document.createElement("div");
    title.className = "toc-title";
    title.textContent = title.textContent || "On this page";
    title.style.fontWeight = "750";
    title.style.fontSize = "0.92rem";
    title.style.color = "var(--fg-muted, rgba(0,0,0,0.66))";
    title.style.textTransform = "uppercase";
    title.style.letterSpacing = "0.04em";
    title.style.margin = "0 0 0.75rem 0";
    container.appendChild(title);

    const tree = buildTree(headings);
    const list = createList(tree, 2);
    container.appendChild(list);
  }

  function setActive(container, id) {
    const links = $all(`.${CFG.tocLinkClass}`, container);
    for (const a of links) {
      const ok = a.getAttribute("data-target-id") === id;
      a.classList.toggle(CFG.activeClass, ok);
      if (ok) {
        a.style.background = "rgba(110,86,207,0.12)";
        a.style.color = "var(--fg, rgba(0,0,0,0.90))";
      } else {
        a.style.background = "transparent";
        a.style.color = "var(--fg-muted, rgba(0,0,0,0.66))";
      }
    }
  }

  function computeActiveHeading(headings) {
    // Fallback if no IntersectionObserver: choose closest heading above viewport
    const top = getHeaderHeight() + CFG.headerOffsetPx + 6;
    let best = null;
    let bestY = -Infinity;

    for (const h of headings) {
      const r = h.getBoundingClientRect();
      const y = r.top - top;
      if (y <= 0 && y > bestY) {
        bestY = y;
        best = h;
      }
    }
    return best || headings[0] || null;
  }

  function observeHeadings(container, headings) {
    if (!CFG.useIntersectionObserver || !("IntersectionObserver" in window)) {
      // Scroll listener fallback
      const onScroll = () => {
        const h = computeActiveHeading(headings);
        if (h) setActive(container, h.id);
      };
      window.addEventListener("scroll", onScroll, { passive: true });
      onScroll();
      return;
    }

    const topMargin = getHeaderHeight() + CFG.headerOffsetPx + 6;

    const io = new IntersectionObserver(
      (entries) => {
        // Choose the visible heading with smallest top distance
        let best = null;
        let bestTop = Infinity;

        for (const e of entries) {
          if (!e.isIntersecting) continue;
          const top = e.boundingClientRect.top;
          if (top >= 0 && top < bestTop) {
            bestTop = top;
            best = e.target;
          }
        }

        if (!best) {
          const fallback = computeActiveHeading(headings);
          if (fallback) setActive(container, fallback.id);
          return;
        }

        setActive(container, best.id);
      },
      {
        root: null,
        // Top margin accounts for sticky header; bottom margin makes next headings activate earlier
        rootMargin: `-${topMargin}px 0px -70% 0px`,
        threshold: [0, 1],
      }
    );

    headings.forEach((h) => io.observe(h));

    // Initial
    const initial = computeActiveHeading(headings);
    if (initial) setActive(container, initial.id);
  }

  function main() {
    const container = $(CFG.tocContainerSelector);
    const content = $(CFG.contentSelector);
    if (!container || !content) return;

    const headings = $all(CFG.headingsSelector, content).filter(shouldInclude);
    if (headings.length < CFG.minHeadings) {
      container.style.display = "none";
      return;
    }

    ensureHeadingIds(headings);
    renderTOC(container, headings);
    observeHeadings(container, headings);
  }

  if (document.readyState === "loading") {
    document.addEventListener("DOMContentLoaded", main, { once: true });
  } else {
    main();
  }
})();
