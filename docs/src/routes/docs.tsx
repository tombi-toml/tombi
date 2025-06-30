import type { RouteSectionProps } from "@solidjs/router";
import { useLocation } from "@solidjs/router";
import { Sidebar } from "~/components/Sidebar";
import { createEffect } from "solid-js";
import Prism from "prismjs";
import { DocNavigation } from "~/components/DocNavigation";
import { setupAnchors } from "~/utils/anchor";

export default function DocumentationLayout(props: RouteSectionProps) {
  const location = useLocation();

  createEffect(() => {
    location.pathname;
    requestAnimationFrame(() => {
      Prism.highlightAll();
      setupAnchors();
    });
  });

  return (
    <div class="flex w-full max-w-[100vw] overflow-x-hidden">
      <Sidebar />
      <main class="flex-1 p-4 mdx-content min-h-screen max-w-full overflow-x-hidden">
        <div class="max-w-full">
          {props.children}
          <DocNavigation />
        </div>
      </main>
    </div>
  );
}
