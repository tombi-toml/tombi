import { useLocation } from "@solidjs/router";
import { createEffect, onCleanup, onMount } from "solid-js";
import { handleAnchorClick, handleInitialHash } from "../utils/anchor-scroll";

/**
 * Component that handles anchor link scrolling and route changes
 * This component should be placed inside the Router context
 */
export function AnchorHandler() {
  const location = useLocation();

  onMount(() => {
    window.addEventListener("hashchange", handleInitialHash);
    document.addEventListener("click", handleAnchorClick);
    onCleanup(() => {
      window.removeEventListener("hashchange", handleInitialHash);
      document.removeEventListener("click", handleAnchorClick);
    });
  });

  // Watch for route changes using createEffect
  createEffect(() => {
    // Access location.pathname to trigger the effect when route changes
    location.pathname;

    // Small delay to ensure DOM is updated after route change
    const timeoutId = setTimeout(() => {
      handleInitialHash();
    }, 50);
    onCleanup(() => clearTimeout(timeoutId));
  });

  return null; // This component doesn't render anything
}
