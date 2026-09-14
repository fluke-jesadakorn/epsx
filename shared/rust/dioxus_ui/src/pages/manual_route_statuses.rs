// @generated from docs/migration/contracts/frontend-live-data.json
// Checked source: keep this deterministic table byte-comparable with the contract.

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(super) enum RouteMigrationStatus {
    Aligned,
    Partial,
    Blocked,
}

impl RouteMigrationStatus {
    pub(super) const fn label(self) -> &'static str {
        match self {
            Self::Aligned => "Migration aligned",
            Self::Partial => "Migration partial",
            Self::Blocked => "Migration blocked",
        }
    }

    pub(super) const fn token(self) -> &'static str {
        match self {
            Self::Aligned => "aligned",
            Self::Partial => "partial",
            Self::Blocked => "blocked",
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(super) struct ManualRouteStatus {
    pub(super) target_route: &'static str,
    pub(super) status: RouteMigrationStatus,
}

pub(super) const MANUAL_ROUTE_STATUSES: &[ManualRouteStatus] = &[
    ManualRouteStatus {
        target_route: "/",
        status: RouteMigrationStatus::Blocked,
    },
    ManualRouteStatus {
        target_route: "/about",
        status: RouteMigrationStatus::Aligned,
    },
    ManualRouteStatus {
        target_route: "/news",
        status: RouteMigrationStatus::Partial,
    },
    ManualRouteStatus {
        target_route: "/news/:slug",
        status: RouteMigrationStatus::Partial,
    },
    ManualRouteStatus {
        target_route: "/auth",
        status: RouteMigrationStatus::Blocked,
    },
    ManualRouteStatus {
        target_route: "/account",
        status: RouteMigrationStatus::Blocked,
    },
    ManualRouteStatus {
        target_route: "/account/credits",
        status: RouteMigrationStatus::Blocked,
    },
    ManualRouteStatus {
        target_route: "/profile",
        status: RouteMigrationStatus::Blocked,
    },
    ManualRouteStatus {
        target_route: "/analytics",
        status: RouteMigrationStatus::Blocked,
    },
    ManualRouteStatus {
        target_route: "/dashboard",
        status: RouteMigrationStatus::Blocked,
    },
    ManualRouteStatus {
        target_route: "/portfolio",
        status: RouteMigrationStatus::Blocked,
    },
    ManualRouteStatus {
        target_route: "/permissions",
        status: RouteMigrationStatus::Blocked,
    },
    ManualRouteStatus {
        target_route: "/chat",
        status: RouteMigrationStatus::Blocked,
    },
    ManualRouteStatus {
        target_route: "/chat/:id",
        status: RouteMigrationStatus::Blocked,
    },
    ManualRouteStatus {
        target_route: "/chat/history",
        status: RouteMigrationStatus::Blocked,
    },
    ManualRouteStatus {
        target_route: "/notifications",
        status: RouteMigrationStatus::Partial,
    },
    ManualRouteStatus {
        target_route: "/developer",
        status: RouteMigrationStatus::Blocked,
    },
    ManualRouteStatus {
        target_route: "/developer/docs",
        status: RouteMigrationStatus::Partial,
    },
    ManualRouteStatus {
        target_route: "/developer/usage",
        status: RouteMigrationStatus::Blocked,
    },
    ManualRouteStatus {
        target_route: "/manual",
        status: RouteMigrationStatus::Partial,
    },
    ManualRouteStatus {
        target_route: "/plans",
        status: RouteMigrationStatus::Blocked,
    },
    ManualRouteStatus {
        target_route: "/payment",
        status: RouteMigrationStatus::Blocked,
    },
    ManualRouteStatus {
        target_route: "/payment/:type/:id",
        status: RouteMigrationStatus::Blocked,
    },
    ManualRouteStatus {
        target_route: "/contact",
        status: RouteMigrationStatus::Partial,
    },
    ManualRouteStatus {
        target_route: "/access-denied",
        status: RouteMigrationStatus::Partial,
    },
    ManualRouteStatus {
        target_route: "/offline",
        status: RouteMigrationStatus::Partial,
    },
    ManualRouteStatus {
        target_route: "/privacy",
        status: RouteMigrationStatus::Partial,
    },
    ManualRouteStatus {
        target_route: "/terms",
        status: RouteMigrationStatus::Partial,
    },
];
