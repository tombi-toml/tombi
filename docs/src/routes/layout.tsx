import type { JSX } from "solid-js";
import { AnchorHandler } from "~/components/AnchorHandler";
import { Header } from "~/components/header/index";

export default function Layout(props: { children: JSX.Element }) {
  return (
    <div class="min-h-screen bg-gray-50 dark:bg-gray-700 text-gray-900 dark:text-gray-100 overflow-y-auto">
      <Header />
      <AnchorHandler />
      {props.children}
    </div>
  );
}
