//! `news` domain subdir — 2 components ported from
//! `apps-old/frontend/components/news/`:
//! - `news_list`   (185 LoC)
//! - `news_detail` (148 LoC)
//!
//! Both are ported as visual stubs with typed prop shapes that
//! the page-level BFF call sites can fill in from the
//! `epsx_client::ServiceClient` calls.

use dioxus::prelude::*;

pub mod news_list;
pub mod news_detail;

pub use news_list::{NewsList, NewsListItem};
pub use news_detail::{NewsDetail, NewsDetailItem};

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn news_module_re_exports_resolve() {
        
        
    }
}
