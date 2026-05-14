mod wallpaper_detail;
pub use wallpaper_detail::WallpaperDetail;

mod explore;
pub use explore::Explore;

mod search;
pub use search::Search;

mod upload;
pub use upload::Upload;

mod profile;
pub use profile::Profile;

mod login;
pub use login::Login;

mod register;
pub use register::Register;

mod settings;
pub use settings::{
    SettingsLayout, SettingsAccount, SettingsAppearance, SettingsDownloads, SettingsNotifications,
};
#[cfg(feature = "desktop")]
pub use settings::{SettingsKeybinds, SettingsSystem};

mod collections;
pub use collections::Collections;

mod collection_detail;
pub use collection_detail::CollectionDetail;

mod home;
pub use home::Home;

mod legal;
pub use legal::*;

mod latest;
pub use latest::Latest;

mod popular;
pub use popular::{PopularGrid, PopularSelection};

mod editorial;
pub use editorial::Editorial;

mod ai_generated;
pub use ai_generated::AiGenerated;

mod live_wallpapers;
pub use live_wallpapers::LiveWallpapers;

mod info;
pub use info::*;

mod public_profile;
pub use public_profile::PublicProfile;

pub mod admin;
pub use admin::{
    AdminLayout,
    dashboard::AdminDashboard,
    dmca::AdminDmca,
    reports::AdminReports,
    appeals::AdminAppeals,
    users::AdminUsers,
};

mod forgot_password;
pub use forgot_password::*;

mod reset_password;
pub use reset_password::*;

mod follows;
pub use follows::{UserFollowers, UserFollowing};

mod appeal;
pub use appeal::Appeal;

mod not_found;
pub use not_found::NotFound;
