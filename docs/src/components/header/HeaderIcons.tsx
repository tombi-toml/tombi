import { createSignal, onMount } from "solid-js";
import { TbMoonFilled, TbSunFilled } from "solid-icons/tb";
import { LinkImageButton } from "../button/LinkImageButton";
import { LinkIconButton } from "../button/LinkIconButton";

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
    <div class="hidden md:flex items-center px-4 space-x-4">
      <LinkIconButton
        id="dark-mode-toggle"
        onClick={toggleDarkMode}
        alt="Toggle dark mode"
        class={`flex items-center justify-center transition-transform duration-300 ease-out forwards ${isDark() ? 'rotate-0' : '-rotate-90'}`}
      >
        {
          isDark()
            ? <TbMoonFilled size={28}/>
            : <TbSunFilled size={28}/>
        }
      </LinkIconButton>
      <LinkImageButton
        href="https://github.com/tombi-toml/tombi"
        src="/github-mark.svg"
        alt="GitHub"
        class="w-6 h-6"
      />
    </div>
  );
}
