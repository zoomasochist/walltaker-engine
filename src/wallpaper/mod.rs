use std::path::Path;
use windows::{
    core::{PCWSTR, HSTRING},
    Win32::{
        System::Com::{
            COINIT_MULTITHREADED, CLSCTX_ALL,
            CoInitializeEx, CoCreateInstance
        },
        UI::Shell::{
            DESKTOP_WALLPAPER_POSITION,
            IDesktopWallpaper, DesktopWallpaper
        }, Foundation::HWND,
    },
};

mod video;
use video::Video;

pub struct Wallpaper {
    video: Video,
    current_media: String,
    old_wallpaper: String,
    old_wallpaper_method: i32,
}

impl Wallpaper {
    pub fn new(hwnds: &[HWND]) -> anyhow::Result<Self> {
        let (old_wallpaper, old_wallpaper_method) = get_old_wallpaper()?;

        Ok(Wallpaper {
            video: Video::new(hwnds),
            current_media: String::new(),
            old_wallpaper,
            old_wallpaper_method,
        })
    }

    pub fn set<T: AsRef<Path>>(&mut self, path: T, method: i32) -> anyhow::Result<()> {
        let path = path.as_ref().to_string_lossy().to_string();

        self.current_media = path.clone();

        #[allow(clippy::case_sensitive_file_extension_comparisons)]
        if path.ends_with(".webm") || path.ends_with(".mp4") {
            self.video.set_video(&path);
            self.video.play();
        } else {
            self.video.pause();

            unsafe {
                CoInitializeEx(None, COINIT_MULTITHREADED)?;
                let idw: IDesktopWallpaper = CoCreateInstance(&DesktopWallpaper, None, CLSCTX_ALL)?;
                idw.SetWallpaper(PCWSTR::null(), &HSTRING::from(path))?;
                idw.SetPosition(DESKTOP_WALLPAPER_POSITION(method))?;
            }
        }

        Ok(())
    }

    pub fn current_media(&self) -> &str {
        &self.current_media
    }

    pub fn set_method(to: i32) -> anyhow::Result<()> {
        unsafe {
            CoInitializeEx(None, COINIT_MULTITHREADED)?;
            let idw: IDesktopWallpaper = CoCreateInstance(&DesktopWallpaper, None, CLSCTX_ALL)?;
            idw.SetPosition(DESKTOP_WALLPAPER_POSITION(to))?;
        }
    
        Ok(())
    }

    pub fn reset(&mut self) -> anyhow::Result<()> {
        let old = self.old_wallpaper.clone();
        self.set(&old, self.old_wallpaper_method)
    }
}

fn get_old_wallpaper() -> anyhow::Result<(String, i32)> {
    unsafe {
        CoInitializeEx(None, COINIT_MULTITHREADED)?;
        let idw: IDesktopWallpaper = CoCreateInstance(&DesktopWallpaper, None, CLSCTX_ALL)?;
        let first_monitor = idw.GetMonitorDevicePathAt(0)?;

        let prev_wallpaper_path = std::path::PathBuf::from(
            idw.GetWallpaper(&first_monitor.to_hstring()?)?.to_string()?);

        let prev_wallpaper_position = idw.GetPosition()?.0;

        Ok((prev_wallpaper_path.to_string_lossy().to_string(), prev_wallpaper_position))
    }
}