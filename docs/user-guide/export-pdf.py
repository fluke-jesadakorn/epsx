"""Export both user manuals with externally installed Python Playwright/Chromium."""
import argparse
from pathlib import Path

from playwright.sync_api import sync_playwright


def main():
    source = Path(__file__).resolve().parent
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--output-dir", type=Path, default=source.parents[1] / "output/pdf")
    parser.add_argument("--chromium", help="Optional existing Chromium executable")
    args = parser.parse_args()
    args.output_dir.mkdir(parents=True, exist_ok=True)
    with sync_playwright() as playwright:
        options = {"headless": True}
        if args.chromium:
            options["executable_path"] = args.chromium
        browser = playwright.chromium.launch(**options)
        try:
            for language, filename in [("th", "manual.html"), ("en", "manual-en.html")]:
                page = browser.new_page()
                try:
                    page.goto((source / filename).as_uri())
                    page.evaluate("() => document.fonts.ready")
                    overflow = page.locator(".page").evaluate_all(
                        "pages => pages.map((p,i) => ({page:i+1, overflow:p.scrollHeight>p.clientHeight}))"
                    )
                    if any(item["overflow"] for item in overflow):
                        raise RuntimeError(f"Page overflow: {overflow}")
                    output = args.output_dir / f"epsx-user-guide-{language}.pdf"
                    page.pdf(path=str(output), print_background=True, prefer_css_page_size=True)
                    print(output)
                finally:
                    page.close()
        finally:
            browser.close()


if __name__ == "__main__":
    main()
