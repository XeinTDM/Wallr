use api::Wallpaper;
use std::sync::{Arc, Mutex};
use std::time::Duration;
use tokio::sync::mpsc;

#[derive(Debug, Clone, PartialEq)]
pub enum PlaybackState {
    Playing,
    Paused(PauseReason),
}

#[derive(Debug, Clone, PartialEq)]
pub enum PauseReason {
    Battery,
    FullscreenApp,
}

#[derive(Debug)]
pub struct EngineState {
    pub current_wallpapers: std::collections::HashMap<String, Wallpaper>, // monitor_name -> Wallpaper
    pub history: Vec<std::collections::HashMap<String, Wallpaper>>,       // Back stack
    pub playback_state: PlaybackState,
}

impl Default for EngineState {
    fn default() -> Self {
        Self {
            current_wallpapers: std::collections::HashMap::new(),
            history: Vec::new(),
            playback_state: PlaybackState::Playing,
        }
    }
}

#[derive(Clone)]
pub struct WallpaperEngine {
    state: Arc<Mutex<EngineState>>,
    command_tx: mpsc::Sender<EngineCommand>,
}

#[derive(Debug, Clone)]
pub enum EngineCommand {
    ApplyWallpaper {
        monitor_name: Option<String>,
        wallpaper: Wallpaper,
    },
    GoBack,
    Tick,
}

impl WallpaperEngine {
    pub fn new() -> (Self, mpsc::Receiver<EngineCommand>) {
        let (tx, rx) = mpsc::channel(100);
        (
            Self {
                state: Arc::new(Mutex::new(EngineState::default())),
                command_tx: tx,
            },
            rx,
        )
    }

    pub fn get_state(&self) -> Arc<Mutex<EngineState>> {
        self.state.clone()
    }

    pub async fn send_command(&self, cmd: EngineCommand) {
        let _ = self.command_tx.send(cmd).await;
    }

    pub fn start_monitor_loop(engine_clone: Self) {
        tokio::spawn(async move {
            loop {
                tokio::time::sleep(Duration::from_secs(5)).await;
                engine_clone.send_command(EngineCommand::Tick).await;
            }
        });
    }
}

#[cfg(target_os = "windows")]
pub mod system_monitor {
    use windows::Win32::Foundation::RECT;
    use windows::Win32::System::Power::{GetSystemPowerStatus, SYSTEM_POWER_STATUS};
    use windows::Win32::UI::WindowsAndMessaging::{
        GetDesktopWindow, GetForegroundWindow, GetSystemMetrics, GetWindowRect, SM_CXSCREEN,
        SM_CYSCREEN,
    };

    pub fn is_on_battery() -> bool {
        unsafe {
            let mut status = SYSTEM_POWER_STATUS::default();
            if GetSystemPowerStatus(&mut status).is_ok() {
                return status.ACLineStatus == 0;
            }
        }
        false
    }

    pub fn is_fullscreen_app_running() -> bool {
        unsafe {
            let fg = GetForegroundWindow();
            let desktop = GetDesktopWindow();
            if fg == desktop || fg.0.is_null() {
                return false;
            }

            let mut rect = RECT::default();
            if GetWindowRect(fg, &mut rect).is_ok() {
                let screen_w = GetSystemMetrics(SM_CXSCREEN);
                let screen_h = GetSystemMetrics(SM_CYSCREEN);

                return rect.left <= 0
                    && rect.top <= 0
                    && rect.right >= screen_w
                    && rect.bottom >= screen_h;
            }
        }
        false
    }
}

pub async fn run_engine_loop(
    state: Arc<Mutex<EngineState>>,
    mut rx: mpsc::Receiver<EngineCommand>,
    desktop_context: dioxus::desktop::DesktopContext,
) {
    // Manage live wallpapers robustly
    let mut active_live_wallpapers: std::collections::HashMap<String, String> =
        std::collections::HashMap::new();

    while let Some(cmd) = rx.recv().await {
        match cmd {
            EngineCommand::ApplyWallpaper {
                monitor_name,
                wallpaper,
            } => {
                let mut s = state.lock().unwrap();
                if !s.current_wallpapers.is_empty() {
                    let current = s.current_wallpapers.clone();
                    s.history.push(current);
                    if s.history.len() > 50 {
                        s.history.remove(0); // keep max 50
                    }
                }

                let m_name = monitor_name.unwrap_or_else(|| "default".to_string());
                s.current_wallpapers
                    .insert(m_name.clone(), wallpaper.clone());

                // Actual apply logic happens here (simplified for desktop engine)
                // In a real scenario we use Tao/Windows API to set per-monitor or spawn Dioxus windows per monitor.
                drop(s);

                let _ = apply_wallpaper_actual(
                    desktop_context.clone(),
                    wallpaper,
                    &mut active_live_wallpapers,
                    m_name,
                )
                .await;
            }
            EngineCommand::GoBack => {
                let mut s = state.lock().unwrap();
                if let Some(prev) = s.history.pop() {
                    s.current_wallpapers = prev.clone();
                    drop(s);
                    for (m_name, wp) in prev {
                        let _ = apply_wallpaper_actual(
                            desktop_context.clone(),
                            wp,
                            &mut active_live_wallpapers,
                            m_name,
                        )
                        .await;
                    }
                }
            }
            EngineCommand::Tick => {
                // Check schedule and system state
                #[cfg(target_os = "windows")]
                {
                    let is_battery = system_monitor::is_on_battery();
                    let is_fullscreen = system_monitor::is_fullscreen_app_running();

                    let mut s = state.lock().unwrap();
                    if is_battery || is_fullscreen {
                        if s.playback_state == PlaybackState::Playing {
                            let reason = if is_battery {
                                PauseReason::Battery
                            } else {
                                PauseReason::FullscreenApp
                            };
                            s.playback_state = PlaybackState::Paused(reason);
                            // TODO: trigger pause on active windows
                        }
                    } else if let PlaybackState::Paused(reason) = &s.playback_state {
                        match reason {
                            _ => {
                                s.playback_state = PlaybackState::Playing;
                                // TODO: trigger resume on active windows
                            }
                        }
                    }
                }
            }
        }
    }
}

pub async fn apply_wallpaper_actual(
    window: dioxus::desktop::DesktopContext,
    wp: Wallpaper,
    active_live_wallpapers: &mut std::collections::HashMap<String, String>,
    monitor_name: String,
) {
    if wp.is_live {
        use dioxus::desktop::{Config, WindowBuilder};

        let path_res = crate::cache::get_cached_wallpaper(&wp.image_url, &wp.id).await;
        let url = if let Ok(path) = path_res {
            let filename = path.file_name().unwrap().to_str().unwrap();
            format!("/cache/{}", filename)
        } else {
            let filename = wp
                .image_url
                .strip_prefix("/assets/uploads/")
                .unwrap_or(&wp.image_url);
            format!("/upload/{}", filename)
        };

        if let Some(current_url) = active_live_wallpapers.get(&monitor_name) {
            if current_url == &url {
                return;
            }
        }
        active_live_wallpapers.insert(monitor_name.clone(), url.clone());

        let dom = dioxus::core::VirtualDom::new_with_props(
            crate::LiveWallpaperView,
            crate::LiveWallpaperProps { url, monitor_name },
        );
        let wb = WindowBuilder::new()
            .with_title("Wallr Live Background")
            .with_decorations(false)
            .with_always_on_bottom(true)
            .with_maximized(true);

        #[cfg(target_os = "linux")]
        {
            use dioxus::desktop::tao::platform::unix::WindowBuilderExtUnix;
            wb = wb.with_window_type(vec![
                dioxus::desktop::tao::platform::unix::WindowType::Desktop,
            ]);
        }

        let cfg = Config::new().with_window(wb);
        window.new_window(dom, cfg);
    } else {
        active_live_wallpapers.remove(&monitor_name);

        let image_url = wp.image_url.clone();
        let wp_id = wp.id.clone();

        let path_res = crate::cache::get_cached_wallpaper(&image_url, &wp_id).await;

        let _ = tokio::task::spawn_blocking(move || {
            if let Ok(cached_path) = path_res {
                let is_supported = cached_path.extension().map_or(false, |ext| {
                    let ext_str = ext.to_string_lossy().to_lowercase();
                    ext_str == "jpg" || ext_str == "jpeg" || ext_str == "png" || ext_str == "bmp"
                });

                let path_to_set = if is_supported {
                    cached_path
                } else {
                    if let Ok(img) = image::open(&cached_path) {
                        let temp_dir = std::env::temp_dir();
                        let temp_path = temp_dir.join(format!("wallr_{}.jpg", wp_id));
                        if img.save(&temp_path).is_ok() {
                            temp_path
                        } else {
                            return;
                        }
                    } else {
                        return;
                    }
                };

                if let Some(path_str) = path_to_set.to_str() {
                    let _ = wallpaper::set_from_path(path_str);
                    let config = crate::config::AppConfig::load();
                    let mode = match config.wallpaper_mode.as_str() {
                        "Center" => wallpaper::Mode::Center,
                        "Crop" => wallpaper::Mode::Crop,
                        "Fit" => wallpaper::Mode::Fit,
                        "Span" => wallpaper::Mode::Span,
                        "Stretch" => wallpaper::Mode::Stretch,
                        "Tile" => wallpaper::Mode::Tile,
                        _ => wallpaper::Mode::Crop,
                    };
                    let _ = wallpaper::set_mode(mode);
                }
            }
        })
        .await;
    }
}
