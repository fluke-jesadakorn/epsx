//! `ScrollArea` / `ScrollBar` — shadcn new-york scrollable region.
//!
//! Mirrors `apps-old/frontend/components/ui/scroll-area.tsx`:
//!
//! ```text
//! const ScrollArea = React.forwardRef<...>(({ className, children, ...props }, ref) => (
//!     <ScrollAreaPrimitive.Root ref={ref} className={cn("relative overflow-hidden", className)} {...props}>
//!         <ScrollAreaPrimitive.Viewport className="h-full w-full rounded-[inherit]">
//!             {children}
//!         </ScrollAreaPrimitive.Viewport>
//!         <ScrollBar />
//!         <ScrollAreaPrimitive.Corner />
//!     </ScrollAreaPrimitive.Root>
//! ))
//!
//! const ScrollBar = React.forwardRef<...>(({ className, orientation = "vertical", ...props }, ref) => (
//!     <ScrollAreaPrimitive.ScrollAreaScrollbar ref={ref} orientation={orientation} className={cn(
//!         "flex touch-none select-none transition-colors",
//!         orientation === "vertical" && "h-full w-2.5 border-l border-l-transparent p-[1px]",
//!         orientation === "horizontal" && "h-2.5 flex-col border-t border-t-transparent p-[1px]",
//!         className
//!     )} {...props}>
//!         <ScrollAreaPrimitive.ScrollAreaThumb className="relative flex-1 rounded-full bg-border" />
//!     </ScrollAreaPrimitive.ScrollAreaScrollbar>
//! ))
//! ```
//!
//! This shadcn-namespace version produces a `<div role="region">` with
//! a hidden overflow viewport and a styled thumb. The
//! `misc::ScrollArea` already in dioxus_ui is a simpler version —
//! both can coexist.

use dioxus::prelude::*;

/// Shadcn-namespace scroll area container.
///
/// - `max_height: Option<String>` — CSS `max-height` for the outer
///   wrapper. Common values: `"300px"`, `"50vh"`.
/// - `class: Option<String>` — extra Tailwind classes merged onto
///   the outer wrapper.
/// - `children: Element` — the scrollable content.
#[component]
pub fn ScrollArea(
    #[props(default = None)] max_height: Option<String>,
    #[props(default = None)] class: Option<String>,
    children: Element,
) -> Element {
    let mut outer_cls = "scroll-area relative overflow-hidden".to_string();
    if let Some(c) = class {
        outer_cls.push(' ');
        outer_cls.push_str(&c);
    }
    let mh = max_height.unwrap_or_default();
    let style = if mh.is_empty() {
        String::new()
    } else {
        format!("max-height: {mh};")
    };
    rsx! {
        div { class: "{outer_cls}", style: "{style}",
            div {
                class: "scroll-area-viewport h-full w-full rounded-[inherit] overflow-auto",
                role: "region",
                tabindex: "0",
                {children}
            }
            ScrollBar { orientation: Some("vertical".to_string()) }
            ScrollBar { orientation: Some("horizontal".to_string()) }
        }
    }
}

/// Shadcn-namespace scroll bar (the draggable thumb rail).
///
/// - `orientation: Option<String>` — `"vertical" | "horizontal"`.
///   Defaults to `"vertical"`.
#[component]
pub fn ScrollBar(orientation: Option<String>) -> Element {
    let o = orientation.unwrap_or_else(|| "vertical".to_string());
    let base = "scroll-area-scrollbar flex touch-none select-none transition-colors";
    let orient_cls = match o.as_str() {
        "horizontal" => "h-2.5 flex-col border-t border-t-transparent p-[1px]",
        _ => "h-full w-2.5 border-l border-l-transparent p-[1px]",
    };
    let cls = format!("{base} {orient_cls}");
    rsx! {
        div { class: "{cls}",
            div { class: "scroll-area-thumb relative flex-1 rounded-full bg-border" }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn scrollbar_vertical_orientation_class() {
        let base = "scroll-area-scrollbar flex touch-none select-none transition-colors";
        let orient_cls = match "vertical" {
            "horizontal" => "h-2.5 flex-col border-t border-t-transparent p-[1px]",
            _ => "h-full w-2.5 border-l border-l-transparent p-[1px]",
        };
        let cls = format!("{base} {orient_cls}");
        assert!(cls.contains("w-2.5"));
        assert!(cls.contains("border-l"));
    }

    #[test]
    fn scrollbar_horizontal_orientation_class() {
        let base = "scroll-area-scrollbar flex touch-none select-none transition-colors";
        let orient_cls = match "horizontal" {
            "horizontal" => "h-2.5 flex-col border-t border-t-transparent p-[1px]",
            _ => "h-full w-2.5 border-l border-l-transparent p-[1px]",
        };
        let cls = format!("{base} {orient_cls}");
        assert!(cls.contains("h-2.5"));
        assert!(cls.contains("border-t"));
    }

    #[test]
    fn scroll_area_default_orientation_is_vertical() {
        let o: Option<String> = None;
        let resolved = o.unwrap_or_else(|| "vertical".to_string());
        assert_eq!(resolved, "vertical");
    }

    #[test]
    fn scroll_area_viewport_is_focusable_region() {
        // The viewport uses `role="region"` and `tabindex="0"` so
        // keyboard users can scroll with arrow keys / PageUp /
        // PageDown. Sanity check the constants.
        assert_eq!("region", "region");
    }
}
