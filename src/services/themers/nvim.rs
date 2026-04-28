use crate::services::themers::{ThemeContext, Themer};
use crate::utils::paths::Paths;
use crate::utils::symlink::Symlink;
use std::fs;

pub struct NvimThemer;

impl Themer for NvimThemer {
    fn apply(&self, _context: &ThemeContext<'_>) -> Result<(), String> {
        let nvim_color_scheme_file_path =
            Paths::user_home()?.join(".config/nvim/lua/plugins/colorscheme.lua");

        if Symlink::exists(&nvim_color_scheme_file_path)? {
            fs::remove_file(&nvim_color_scheme_file_path).map_err(|e| {
                format!(
                    "Could not remove {}: {e}",
                    nvim_color_scheme_file_path.display()
                )
            })?;
        }

        let theme_color_scheme_file_path = Paths::current_theme()?.join("nvim-colorscheme.lua");

        if !fs::exists(&theme_color_scheme_file_path).map_err(|e| {
            format!(
                "Could not check existence of {}: {e}",
                theme_color_scheme_file_path.display()
            )
        })? {
            return Ok(());
        }

        Symlink::create(&theme_color_scheme_file_path, &nvim_color_scheme_file_path)
    }
}
