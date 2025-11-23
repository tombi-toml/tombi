import type { RouteSectionProps } from "@solidjs/router";
import { useLocation } from "@solidjs/router";
import Prism from "prismjs";
import { createEffect } from "solid-js";
import { DocNavigation } from "~/components/DocNavigation";
import { Sidebar } from "~/components/Sidebar";
import { setupAnchors } from "~/utils/anchor";

export default function DocumentationLayout(props: RouteSectionProps) {
  const location = useLocation();
  let mainRef: HTMLElement | undefined;

  createEffect(() => {
    location.pathname;
    requestAnimationFrame(() => {
      Prism.highlightAll();
      setupAnchors();
      // Focus on main content after page transition
      if (mainRef) {
        mainRef.focus();
      }
    });
  });

  return (
    <div class="flex w-full max-w-[100vw] overflow-x-hidden">
      <Sidebar />
      <main
        ref={mainRef}
        tabindex="-1"
        class="flex-1 p-4 mdx-content min-h-screen max-w-full overflow-x-hidden outline-none"
      >
        <div class="max-w-full">
          {props.children}
          <DocNavigation />
        </div>
      </main>
    </div>
  );
}
