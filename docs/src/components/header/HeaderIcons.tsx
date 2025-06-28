import { createSignal, onMount } from "solid-js";
import { TbBrandGithubFilled, TbMoonFilled, TbSunFilled } from "solid-icons/tb";
import { LinkIconButton } from "../button/LinkIconButton";
import { IconButton } from "../button/IconButton";

export function HeaderIcons() {
  const [isDark, setIsDark] = createSignal(false);
  const [rotation, setRotation] = createSignal(0);

  onMount(() => {
    if (typeof window !== "undefined") {
      const darkModePreference = window.matchMedia(
        "(prefers-color-scheme: dark)",
      ).matches;
      const storedTheme = localStorage.getItem("theme");
      setIsDark(storedTheme === "dark" || (!storedTheme && darkModePreference));
      document.documentElement.classList.toggle("dark", isDark());
    }
  });

  const toggleDarkMode = () => {
    const newDarkMode = !isDark();
    setIsDark(newDarkMode);
    if (rotation() % 360 === 0 && newDarkMode) {
      setRotation(rotation() + 180);
    }
    setRotation((rotation() + 180) % 36000);
    localStorage.setItem("theme", newDarkMode ? "dark" : "light");
    document.documentElement.classList.toggle("dark", newDarkMode);
  };

  return (
    <div class="hidden sm:flex items-center md:pl-4 sm:pl-0 pr-4 space-x-1">
      <IconButton
        id="dark-mode-toggle"
        onClick={toggleDarkMode}
        alt="Toggle dark mode"
        class="flex items-center justify-center transition-transform duration-300 ease-out forwards"
        style={`transform: rotate(${rotation()}deg)`}
      >
        {isDark() ? <TbMoonFilled size={28} /> : <TbSunFilled size={28} />}
      </IconButton>
      <LinkIconButton
        href="https://x.com/tombi_toml"
        alt="X (Twitter)"
        class="w-6 h-6"
      >
        <svg
          xmlns="http://www.w3.org/2000/svg"
          viewBox="0 0 512 512"
          aria-label="X (Twitter)"
        >
          <title>X (Twitter)</title>
          <path
            fill="currentColor"
            d="M389.2 48h70.6L305.6 224.2 487 464H345L233.7 318.6 106.5 464H35.8L200.7 275.5 26.8 48H172.4L272.9 180.9 389.2 48zM364.4 421.8h39.1L151.1 88h-42L364.4 421.8z"
          />
        </svg>
      </LinkIconButton>
      <LinkIconButton
        href="https://github.com/tombi-toml/tombi"
        alt="GitHub"
        class="w-6 h-6"
      >
        <TbBrandGithubFilled size={28} aria-label="GitHub" />
      </LinkIconButton>
    </div>
  );
}
