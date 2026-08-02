import type { RouteSectionProps } from "@solidjs/router";
import { useLocation } from "@solidjs/router";
import { createEffect, onCleanup, onMount } from "solid-js";
import { DocNavigation } from "~/components/DocNavigation";
import { Sidebar } from "~/components/Sidebar";
import { setupAnchorCopyHandling, setupAnchors } from "~/utils/anchor";

export default function DocumentationLayout(props: RouteSectionProps) {
  const location = useLocation();
  let mainRef: HTMLElement | undefined;

  onMount(() => {
    const cleanupAnchorCopyHandling = setupAnchorCopyHandling();
    onCleanup(cleanupAnchorCopyHandling);
  });

  createEffect(() => {
    location.pathname;
    const frameId = requestAnimationFrame(() => {
      setupAnchors();
      // Focus on main content after page transition
      if (mainRef) {
        mainRef.focus();
      }
    });
    onCleanup(() => cancelAnimationFrame(frameId));
  });

  return (
    <div class="flex w-full max-w-[100vw]">
      <Sidebar />
      <main
        ref={mainRef}
        tabindex="-1"
        class="flex-1 min-w-0 p-4 mdx-content min-h-screen max-w-full outline-none"
      >
        <div class="max-w-full">
          {props.children}
          <DocNavigation />
        </div>
      </main>
    </div>
  );
}
