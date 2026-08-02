// @refresh reload
import { mount, StartClient } from "@solidjs/start/client";

const app = document.getElementById("app");
if (!app) throw new Error("Failed to find app element");

mount(() => <StartClient />, app);
