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

  maybeRedirectByLanguageOnFirstVisit();
  bindLanguageChoicePersistence();

  const menu = document.querySelector("[data-menu]");
  const toggle = document.querySelector("[data-menu-toggle]");
  const langToggle = document.querySelector("[data-lang-toggle]");
  const langPopover = document.querySelector("[data-lang-popover]");
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
})();
