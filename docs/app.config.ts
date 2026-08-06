import { defineConfig } from "@solidjs/start/config";
/* @ts-expect-error */
import pkg from "@vinxi/plugin-mdx";
import remarkGfm from "remark-gfm";
import unocssPlugin from "unocss/vite";
import { remarkBaseUrl } from "./src/remark/base-url";
import { remarkCode } from "./src/remark/code";
import { remarkHeadingAnchor } from "./src/remark/heading-anchor";
import { remarkPageHeading } from "./src/remark/page-heading";

const { default: mdx } = pkg;

export default defineConfig({
  extensions: ["mdx", "md"],
  ssr: true,
  server: {
    preset: "static",
    baseURL: process.env.BASE_URL,
    prerender: {
      crawlLinks: true,
      failOnError: true,
    },
  },
  vite: {
    base: process.env.BASE_URL,
    optimizeDeps: {
      esbuildOptions: {
        target: ["es2020", "edge88", "firefox78", "chrome87", "safari14.1"],
      },
    },
    plugins: [
      mdx.withImports({})({
        jsx: true,
        jsxImportSource: "solid-js",
        providerImportSource: "solid-mdx",
        remarkPlugins: [
          [remarkGfm, { tablePipeAlign: false }],
          remarkBaseUrl,
          remarkCode,
          remarkPageHeading, // Add page heading with meta tags
          remarkHeadingAnchor,
        ],
      }),
      unocssPlugin(),
    ],
    build: {
      minify: true,
    },
  },
});
