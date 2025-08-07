// @refresh reload
import { mount, StartClient } from "@solidjs/start/client";
import { setupAnchorHandling } from "./utils/anchor-scroll";

const app = document.getElementById("app");
if (!app) throw new Error("Failed to find app element");

// Set up anchor link handling
setupAnchorHandling();

mount(() => <StartClient />, app);
