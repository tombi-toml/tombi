import "prismjs";
import "prismjs/components/prism-toml";
import "prismjs/components/prism-bash";
import "prismjs/components/prism-json";
import { MetaProvider } from "@solidjs/meta";

import { Router } from "@solidjs/router";
import { FileRoutes } from "@solidjs/start/router";
import Prism from "prismjs";
import { onMount, Suspense } from "solid-js";
import { MDXProvider } from "solid-mdx";
import * as components from "~/components";
import "virtual:uno.css";
import "./app.css";
import Layout from "./routes/layout";

export default function App() {
  onMount(() => {
    Prism.highlightAll();
  });

  return (
    <Router
      base={import.meta.env.BASE_URL || undefined}
      root={(props) => (
        <MetaProvider>
          <MDXProvider components={components}>
            <Layout>
              <main class="flex-1 mt-20 pt-0">
                <div class="max-w-7xl mx-auto md:px-6 lg:px-8">
                  <Suspense
                    fallback={<div class="text-center">Loading...</div>}
                  >
                    {props.children}
                  </Suspense>
                </div>
              </main>
            </Layout>
          </MDXProvider>
        </MetaProvider>
      )}
    >
      <FileRoutes />
    </Router>
  );
}
