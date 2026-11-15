import type { JSX } from "solid-js";
import { AnchorHandler } from "~/components/AnchorHandler";
import { Header } from "~/components/header/index";

export default function Layout(props: { children: JSX.Element }) {
  return (
    <div class="min-h-screen bg-gray-50 dark:bg-gray-800 text-gray-900 dark:text-gray-100">
      <link rel="stylesheet" href="/tombi/common/v1/version-picker.css" />
      <script defer src="/tombi/common/v1/version-picker.js" />
      <Header />
      <AnchorHandler />
      {props.children}
    </div>
  );
}
