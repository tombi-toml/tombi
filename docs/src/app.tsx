import { MetaProvider } from "@solidjs/meta";

import { Router } from "@solidjs/router";
import { FileRoutes } from "@solidjs/start/router";
import { Suspense } from "solid-js";
import { MDXProvider } from "solid-mdx";
import * as components from "~/components";
import "virtual:uno.css";
import "./app.css";
import Layout from "./routes/layout";

export default function App() {
  return (
    <Router
      base={import.meta.env.SERVER_BASE_URL || undefined}
      root={(props) => (
        <MetaProvider>
          <MDXProvider components={components}>
            <Layout>
              <main class="flex-1 mt-20 pt-0">
                <div class="max-w-7xl mx-auto">
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
