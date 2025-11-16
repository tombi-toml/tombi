// @refresh reload
import { createHandler, StartServer } from "@solidjs/start/server";

const TITLE = "Tombi - TOML Toolkit";
const DESCRIPTION =
  "a powerful toolkit that provides a TOML Formatter, Linter, and Language Server to help you maintain clean and consistent TOML files.";
const URL = "https://tombi.dev/";
const IMAGE = `${import.meta.env.BASE_URL}/ogp.png`;

export default createHandler(() => (
  <StartServer
    document={({ assets, children, scripts }) => (
      <html lang="en">
        <head>
          <meta charset="utf-8" />
          <meta name="viewport" content="width=device-width, initial-scale=1" />
          <link rel="icon" href={`${import.meta.env.BASE_URL}/favicon.ico`} />

          {/* Primary Meta Tags */}
          <meta name="title" content={TITLE} />
          <meta name="description" content={DESCRIPTION} />

          {/* Open Graph / Facebook */}
          <meta property="og:type" content="website" />
          <meta property="og:url" content={URL} />
          <meta property="og:title" content={TITLE} />
          <meta property="og:description" content={DESCRIPTION} />
          <meta property="og:image" content={IMAGE} />

          {/* Twitter */}
          <meta property="twitter:card" content="summary_large_image" />
          <meta property="twitter:url" content={URL} />
          <meta property="twitter:title" content={TITLE} />
          <meta property="twitter:description" content={DESCRIPTION} />
          <meta property="twitter:image" content={IMAGE} />

          {assets}
        </head>
        <body class="m-0 p-0">
          <div id="app">{children}</div>
          {scripts}
        </body>
      </html>
    )}
  />
));
