import { TbMoonFilled, TbSunFilled } from "solid-icons/tb";
import { createSignal, onCleanup, onMount } from "solid-js";
import { IconButton } from "../button/IconButton";

export function HeaderThemeToggle(props: { mobile?: boolean }) {
  const [isDark, setIsDark] = createSignal(false);
  const [rotation, setRotation] = createSignal(0);

  onMount(() => {
    const storedTheme = localStorage.getItem("theme");
    const dark =
      storedTheme === "dark" ||
      (!storedTheme &&
        window.matchMedia("(prefers-color-scheme: dark)").matches);
    document.documentElement.classList.toggle("dark", dark);
    setIsDark(dark);

    const syncTheme = () => {
      setIsDark(document.documentElement.classList.contains("dark"));
      setRotation((current) => (current + 180) % 36000);
    };
    document.addEventListener("tombi:themechange", syncTheme);
    onCleanup(() =>
      document.removeEventListener("tombi:themechange", syncTheme),
    );
  });

  const toggleTheme = () => {
    const dark = !document.documentElement.classList.contains("dark");
    document.documentElement.classList.toggle("dark", dark);
    localStorage.setItem("theme", dark ? "dark" : "light");
    document.dispatchEvent(new Event("tombi:themechange"));
  };

  return (
    <IconButton
      id={props.mobile ? undefined : "dark-mode-toggle"}
      onClick={toggleTheme}
      alt="Toggle dark mode"
      class={
        props.mobile
          ? "tombi-mobile-social-link flex items-center justify-center w-[2.8rem] h-[2.8rem] transition-transform duration-300 ease-out"
          : "hidden md:flex items-center justify-center transition-transform duration-300 ease-out forwards"
      }
      style={`transform: rotate(${rotation()}deg)`}
    >
      {isDark() ? <TbMoonFilled size={28} /> : <TbSunFilled size={28} />}
    </IconButton>
  );
}
