import init, { format } from "../pkg";

await init();

document.getElementById("run").onclick = async () => {
    const src = document.getElementById("src").value;
    const out = document.getElementById("out");
    try {
        const formatted = await format(src);
        out.textContent = formatted;
    } catch (e) {
        out.textContent = "⚠️ " + e;
    }
};
