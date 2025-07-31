import { useLocation } from "@solidjs/router";
import { onMount, createEffect } from "solid-js";
import { handleInitialHash } from "../utils/anchor-scroll";

/**
 * Component that handles anchor link scrolling and route changes
 * This component should be placed inside the Router context
 */
export function AnchorHandler() {
  const location = useLocation();

  onMount(() => {
    // Handle initial route load
    setTimeout(() => {
      handleInitialHash();
    }, 50);
  });

  // Watch for route changes using createEffect
  createEffect(() => {
    // Access location.pathname to trigger the effect when route changes
    location.pathname;

    // Small delay to ensure DOM is updated after route change
    setTimeout(() => {
      handleInitialHash();
    }, 50);
  });

  return null; // This component doesn't render anything
}
