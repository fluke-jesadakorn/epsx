// Served only from development DX bundles; no Rust rebuild for public CSS edits.
async function updateStyles() {
    try {
        const response = await fetch(new URL("./epsx-dev-live-css.json", import.meta.url), { cache: "no-store" });
        if (!response.ok) return;
        const manifest = await response.json();
        for (const link of document.querySelectorAll('link[rel="stylesheet"]')) {
            const original = link.dataset.epsxCssSource || new URL(link.href).pathname;
            const asset = manifest.styles[original];
            if (!asset || link.dataset.epsxCssVersion === asset.version) continue;
            const next = link.cloneNode();
            next.dataset.epsxCssSource = original;
            next.dataset.epsxCssVersion = asset.version;
            next.href = new URL(asset.file, import.meta.url).href + "?v=" + asset.version;
            await new Promise((resolve) => {
                next.onload = () => { link.remove(); resolve(); };
                next.onerror = () => { next.remove(); resolve(); };
                link.after(next);
            });
        }
    } catch (error) {
        console.debug("Dev CSS update pending", error);
    } finally {
        setTimeout(updateStyles, 1000);
    }
}
updateStyles();
