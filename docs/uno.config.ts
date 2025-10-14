import { defineConfig, presetTypography, presetUno } from "unocss";
import breakpoints from "./breakpoints.json" with { type: "json" };

export default defineConfig({
  presets: [presetUno(), presetTypography()],
  theme: {
    breakpoints: {
      sm: `${breakpoints.sm}px`,
      md: `${breakpoints.md}px`,
      lg: `${breakpoints.lg}px`,
      xl: `${breakpoints.xl}px`,
    },
    colors: {
      tombi: {
        primary: "#000066", // Primary color
        50: "#E6E6FF",
        100: "#CCCCFF",
        200: "#9999FF",
        300: "#6666FF",
        400: "#3333FF",
        500: "#0000FF",
        600: "#0000CC",
        700: "#000099",
        800: "#000066",
        900: "#000066", // Primary color
        brown: "#9C4221",
        focus: "rgba(255,255,255, 0.8)",
      },
    },
    animation: {
      keyframes: {
        "spin-fast":
          "{from{transform:rotate(0deg)}to{transform:rotate(360deg)}}",
        shake:
          "{0%,100%{transform:rotate(0deg)}25%{transform:rotate(-10deg)}75%{transform:rotate(10deg)}}",
      },
      durations: {
        "spin-fast": "0.5s",
        shake: "0.7s",
      },
      timingFns: {
        "spin-fast": "linear",
        shake: "ease-in-out",
      },
      counts: {
        "spin-fast": "infinite",
        shake: "infinite",
      },
    },
  },
  shortcuts: {
    "btn-focus":
      "focus:outline-none focus-visible:ring-2 focus-visible:ring-tombi-focus transition-colors focus:rounded-lg",
  },
});
