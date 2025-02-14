import { createSignal, onMount } from "solid-js";

export function HeaderIcons() {
  const [isDark, setIsDark] = createSignal(false);

  onMount(() => {
    if (typeof window !== 'undefined') {
      const darkModePreference = window.matchMedia('(prefers-color-scheme: dark)').matches;
      const storedTheme = localStorage.getItem('theme');
      setIsDark(storedTheme === 'dark' || (!storedTheme && darkModePreference));
      document.documentElement.classList.toggle('dark', isDark());
    }
  });

  const toggleDarkMode = () => {
    const newDarkMode = !isDark();
    setIsDark(newDarkMode);
    localStorage.setItem('theme', newDarkMode ? 'dark' : 'light');
    document.documentElement.classList.toggle('dark', newDarkMode);
  };

  return (
    <div class="hidden md:flex items-center px-4 space-x-4 flex-shrink-0">
      <button
        id="dark-mode-toggle"
        onClick={toggleDarkMode}
        class="text-white hover:text-white/80 transition-colors bg-transparent border-0 btn-focus"
        aria-label="Toggle dark mode"
      >
        {isDark() ? (
          <img
            src="/dark-mode-icon.svg"
            alt="Dark mode"
            class="w-6 h-6"
          />
        ) : (
          <img
            src="/light-mode-icon.svg"
            alt="Light mode"
            class="w-6 h-6"
          />
        )}
      </button>
      <a
        href="https://github.com/tombi-toml/tombi"
        target="_blank"
        rel="noopener noreferrer"
        class="text-white hover:text-white/80 transition-colors no-underline btn-focus"
        aria-label="GitHub repository"
      >
        <img
          src="/github-mark.svg"
          alt="GitHub"
          class="w-6 h-6"
        />
      </a>
    </div>
  );
}
