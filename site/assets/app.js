(() => {
  const LANGUAGE_REDIRECT_KEY = "agent_world_pages_lang_redirect_done_v1";
  const LANGUAGE_MANUAL_CHOICE_KEY = "agent_world_pages_lang_manual_choice_v1";

  const safeGetStorage = (key) => {
    try {
      return window.localStorage.getItem(key);
    } catch {
      return null;
    }
  };

  const safeSetStorage = (key, value) => {
    try {
      window.localStorage.setItem(key, value);
    } catch {
      return;
    }
  };

  const resolvePreferredLanguage = () => {
    const browserLanguages = Array.isArray(window.navigator.languages)
      ? window.navigator.languages
      : [];
    const firstLanguage = browserLanguages.find(
      (lang) => typeof lang === "string" && lang.trim().length > 0,
    );
    return String(firstLanguage || window.navigator.language || "").toLowerCase();
  };

  const isChineseEntryPath = (pathname) => {
    const onEnglishPage = /\/en\/(?:index\.html)?$/.test(pathname);
    if (onEnglishPage) {
      return false;
    }
    return /\/(?:index\.html)?$/.test(pathname);
  };

  const toEnglishEntryPath = (pathname) => {
    if (pathname.endsWith("/index.html")) {
      return `${pathname.slice(0, -"index.html".length)}en/`;
    }
    if (pathname.endsWith("/")) {
      return `${pathname}en/`;
    }
    return `${pathname}/en/`;
  };

  const maybeRedirectByLanguageOnFirstVisit = () => {
    const manualChoice = safeGetStorage(LANGUAGE_MANUAL_CHOICE_KEY);
    if (manualChoice === "zh" || manualChoice === "en") {
      return;
    }

    if (safeGetStorage(LANGUAGE_REDIRECT_KEY) === "1") {
      return;
    }

    safeSetStorage(LANGUAGE_REDIRECT_KEY, "1");

    const preferredLanguage = resolvePreferredLanguage();
    const prefersEnglish = preferredLanguage.startsWith("en");
    if (!prefersEnglish) {
      return;
    }

    const { pathname, search, hash } = window.location;
    if (!isChineseEntryPath(pathname)) {
      return;
    }

    const targetPath = toEnglishEntryPath(pathname);
    window.location.replace(`${targetPath}${search}${hash}`);
  };

  const bindLanguageChoicePersistence = () => {
    document.querySelectorAll("[data-lang-choice]").forEach((link) => {
      link.addEventListener("click", () => {
        const choice = link.getAttribute("data-lang-choice");
        if (choice === "zh" || choice === "en") {
          safeSetStorage(LANGUAGE_MANUAL_CHOICE_KEY, choice);
          safeSetStorage(LANGUAGE_REDIRECT_KEY, "1");
        }
      });
    });
  };

  const bindSectionReveal = () => {
    const revealNodes = Array.from(document.querySelectorAll("[data-reveal]"));
    if (!revealNodes.length) {
      return;
    }

    const revealAll = () => {
      revealNodes.forEach((node) => node.classList.add("revealed"));
    };

    if (window.matchMedia("(prefers-reduced-motion: reduce)").matches) {
      revealAll();
      return;
    }

    const observer = new IntersectionObserver(
      (entries, obs) => {
        entries.forEach((entry) => {
          if (!entry.isIntersecting) {
            return;
          }
          entry.target.classList.add("revealed");
          obs.unobserve(entry.target);
        });
      },
      {
        threshold: 0.18,
      },
    );

    revealNodes.forEach((node) => observer.observe(node));

    window.setTimeout(() => {
      revealAll();
      observer.disconnect();
    }, 1800);
  };

  const bindCounters = () => {
    const counters = Array.from(document.querySelectorAll("[data-counter-target]"));
    if (!counters.length) {
      return;
    }

    const prefersReducedMotion = window.matchMedia("(prefers-reduced-motion: reduce)").matches;

    const renderFinal = (node) => {
      const target = Number(node.getAttribute("data-counter-target") || "0");
      node.textContent = String(Math.max(0, Math.round(target)));
    };

    if (prefersReducedMotion) {
      counters.forEach(renderFinal);
      return;
    }

    const animate = (node) => {
      const target = Number(node.getAttribute("data-counter-target") || "0");
      const safeTarget = Math.max(0, Math.round(target));
      const startedAt = performance.now();
      const duration = Math.min(1400, 520 + safeTarget * 20);

      const tick = (now) => {
        const progress = Math.min(1, (now - startedAt) / duration);
        const eased = 1 - Math.pow(1 - progress, 3);
        node.textContent = String(Math.round(safeTarget * eased));
        if (progress < 1) {
          window.requestAnimationFrame(tick);
        }
      };

      window.requestAnimationFrame(tick);
    };

    const observer = new IntersectionObserver(
      (entries, obs) => {
        entries.forEach((entry) => {
          if (!entry.isIntersecting) {
            return;
          }
          animate(entry.target);
          obs.unobserve(entry.target);
        });
      },
      {
        threshold: 0.4,
      },
    );

    counters.forEach((node) => observer.observe(node));

    window.setTimeout(() => {
      counters.forEach((node) => {
        if (node.textContent === "0") {
          const target = Number(node.getAttribute("data-counter-target") || "0");
          node.textContent = String(Math.max(0, Math.round(target)));
        }
      });
      observer.disconnect();
    }, 2200);
  };

  const bindActiveNav = () => {
    const nav = document.querySelector("[data-section-nav]");
    if (!nav) {
      return;
    }

    const links = Array.from(nav.querySelectorAll("a[href^='#']"));
    if (!links.length) {
      return;
    }

    const linkMap = new Map();
    const sections = [];

    links.forEach((link) => {
      const id = link.getAttribute("href")?.slice(1);
      if (!id) {
        return;
      }
      const section = document.getElementById(id);
      if (!section) {
        return;
      }
      linkMap.set(section, link);
      sections.push(section);
    });

    const setActiveLink = (link) => {
      links.forEach((node) => {
        if (node === link) {
          node.classList.add("active");
        } else {
          node.classList.remove("active");
        }
      });
    };

    if (links[0]) {
      setActiveLink(links[0]);
    }

    const observer = new IntersectionObserver(
      (entries) => {
        const visible = entries
          .filter((entry) => entry.isIntersecting)
          .sort((first, second) => second.intersectionRatio - first.intersectionRatio);
        if (!visible.length) {
          return;
        }
        const current = visible[0].target;
        const link = linkMap.get(current);
        if (link) {
          setActiveLink(link);
        }
      },
      {
        rootMargin: "-28% 0px -54% 0px",
        threshold: [0.2, 0.35, 0.5],
      },
    );

    sections.forEach((section) => observer.observe(section));
  };

  const bindTimelineFilters = () => {
    const controls = document.querySelector("[data-timeline-controls]");
    const timeline = document.querySelector("[data-timeline-group]");
    if (!controls || !timeline) {
      return;
    }

    const buttons = Array.from(controls.querySelectorAll("[data-timeline-filter]"));
    const items = Array.from(timeline.querySelectorAll("[data-timeline-state]"));
    if (!buttons.length || !items.length) {
      return;
    }

    const applyFilter = (filter) => {
      items.forEach((item) => {
        const state = item.getAttribute("data-timeline-state") || "";
        const visible = filter === "all" || state === filter;
        item.setAttribute("data-hidden", visible ? "false" : "true");
      });

      buttons.forEach((button) => {
        const isActive = button.getAttribute("data-timeline-filter") === filter;
        button.classList.toggle("is-active", isActive);
        button.setAttribute("aria-pressed", isActive ? "true" : "false");
      });
    };

    buttons.forEach((button) => {
      button.addEventListener("click", () => {
        const filter = button.getAttribute("data-timeline-filter") || "all";
        applyFilter(filter);
      });
    });

    applyFilter("all");
  };

  const bindStoryPathHighlight = () => {
    const steps = Array.from(document.querySelectorAll("[data-story-step]"));
    if (!steps.length) {
      return;
    }

    const setActiveStep = (active) => {
      steps.forEach((step) => {
        step.classList.toggle("is-active", step === active);
      });
    };

    if (window.matchMedia("(prefers-reduced-motion: reduce)").matches) {
      if (steps[0]) {
        setActiveStep(steps[0]);
      }
      return;
    }

    const observer = new IntersectionObserver(
      (entries) => {
        const visible = entries
          .filter((entry) => entry.isIntersecting)
          .sort((first, second) => second.intersectionRatio - first.intersectionRatio);
        if (!visible.length) {
          return;
        }
        setActiveStep(visible[0].target);
      },
      {
        threshold: [0.35, 0.5, 0.7],
        rootMargin: "-10% 0px -18% 0px",
      },
    );

    steps.forEach((step) => observer.observe(step));

    if (steps[0]) {
      setActiveStep(steps[0]);
    }
  };

  const bindProofSwitcher = () => {
    const controls = document.querySelector("[data-proof-controls]");
    if (!controls) {
      return;
    }

    const buttons = Array.from(controls.querySelectorAll("[data-proof-tab]"));
    const panels = Array.from(document.querySelectorAll("[data-proof-code][data-proof-panel]"));
    const events = Array.from(document.querySelectorAll("[data-proof-event]"));

    if (!buttons.length || !panels.length || !events.length) {
      return;
    }

    const applyTab = (tab) => {
      buttons.forEach((button) => {
        const isActive = button.getAttribute("data-proof-tab") === tab;
        button.classList.toggle("is-active", isActive);
        button.setAttribute("aria-pressed", isActive ? "true" : "false");
      });

      panels.forEach((panel) => {
        const visible = panel.getAttribute("data-proof-panel") === tab;
        panel.setAttribute("data-proof-visible", visible ? "true" : "false");
      });

      events.forEach((item) => {
        const visible = item.getAttribute("data-proof-event") === tab;
        item.setAttribute("data-proof-visible", visible ? "true" : "false");
      });
    };

    buttons.forEach((button, index) => {
      button.addEventListener("click", () => {
        const tab = button.getAttribute("data-proof-tab") || "minimal";
        applyTab(tab);
      });

      button.addEventListener("keydown", (event) => {
        if (event.key !== "ArrowRight" && event.key !== "ArrowLeft") {
          return;
        }

        event.preventDefault();
        const delta = event.key === "ArrowRight" ? 1 : -1;
        const nextIndex = (index + delta + buttons.length) % buttons.length;
        const nextButton = buttons[nextIndex];
        nextButton.focus();
        const tab = nextButton.getAttribute("data-proof-tab") || "minimal";
        applyTab(tab);
      });
    });

    applyTab("minimal");
  };

  maybeRedirectByLanguageOnFirstVisit();
  bindLanguageChoicePersistence();

  const menu = document.querySelector("[data-menu]");
  const toggle = document.querySelector("[data-menu-toggle]");
  const langToggle = document.querySelector("[data-lang-toggle]");
  const langPopover = document.querySelector("[data-lang-popover]");
  const langItems = Array.from(document.querySelectorAll("[data-lang-item]"));
  const yearNode = document.querySelector("[data-year]");

  if (yearNode) {
    yearNode.textContent = String(new Date().getFullYear());
  }

  if (menu && toggle) {
    toggle.addEventListener("click", () => {
      const opened = menu.getAttribute("data-open") === "true";
      menu.setAttribute("data-open", opened ? "false" : "true");
    });

    menu.querySelectorAll("a").forEach((link) => {
      link.addEventListener("click", () => {
        menu.setAttribute("data-open", "false");
      });
    });
  }

  if (langToggle && langPopover) {
    const closePopover = () => {
      langPopover.setAttribute("data-open", "false");
      langToggle.setAttribute("aria-expanded", "false");
    };

    const openPopover = () => {
      langPopover.setAttribute("data-open", "true");
      langToggle.setAttribute("aria-expanded", "true");
    };

    langToggle.addEventListener("click", () => {
      const opened = langPopover.getAttribute("data-open") === "true";
      if (opened) {
        closePopover();
      } else {
        openPopover();
      }
    });

    langToggle.addEventListener("keydown", (event) => {
      if (event.key === "ArrowDown" || event.key === "Enter" || event.key === " ") {
        event.preventDefault();
        openPopover();
        if (langItems[0]) {
          langItems[0].focus();
        }
      }
    });

    langItems.forEach((item, index) => {
      item.addEventListener("keydown", (event) => {
        if (event.key === "Escape") {
          event.preventDefault();
          closePopover();
          langToggle.focus();
          return;
        }

        if (event.key === "ArrowDown") {
          event.preventDefault();
          const next = langItems[index + 1] || langItems[0];
          next.focus();
          return;
        }

        if (event.key === "ArrowUp") {
          event.preventDefault();
          const previous = langItems[index - 1] || langItems[langItems.length - 1];
          previous.focus();
        }
      });
    });

    document.addEventListener("click", (event) => {
      const target = event.target;
      if (!(target instanceof Node)) {
        return;
      }
      if (langToggle.contains(target) || langPopover.contains(target)) {
        return;
      }
      closePopover();
    });

    document.addEventListener("keydown", (event) => {
      if (event.key === "Escape") {
        closePopover();
      }
    });

    langPopover.querySelectorAll("a").forEach((link) => {
      link.addEventListener("click", () => {
        closePopover();
      });
    });
  }

  bindSectionReveal();
  bindCounters();
  bindActiveNav();
  bindTimelineFilters();
  bindStoryPathHighlight();
  bindProofSwitcher();
})();
