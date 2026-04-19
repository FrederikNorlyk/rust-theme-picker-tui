use clap::{Parser, Subcommand};
use theme_picker::services::theme_service::ThemeService;

#[derive(Parser)]
#[command(name = "norlyk", about = "Norlyk settings manager", version)]
struct Args {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    Theme {
        name: String,
    },
    Wallpaper {
        #[command(subcommand)]
        action: WallpaperAction,
    },
}

#[derive(Subcommand)]
enum WallpaperAction {
    Reload,
}

fn main() {
    let args = Args::parse();

    match args.command {
        Commands::Theme { name } => {
            let themes = ThemeService::get_available_themes().unwrap_or_else(|e| {
                eprintln!("Could not get themes: {e}");
                Vec::new()
            });

            let Some(theme) = themes.iter().find(|theme| theme.name.eq(&name)) else {
                eprintln!("Could not get theme: {name}");
                eprintln!("Available themes:");
                for theme in themes {
                    eprintln!(" - {}", theme.name);
                }
                return;
            };

            match ThemeService::set_current_theme(theme) {
                Ok(()) => println!("The theme was set successfully"),
                Err(e) => eprintln!("Error setting theme: {e}"),
            }
        }
        Commands::Wallpaper { action } => match action {
            WallpaperAction::Reload => match ThemeService::change_wallpaper() {
                Ok(()) => println!("The wallpaper was reloaded"),
                Err(e) => eprintln!("Error reloading wallpaper: {e}"),
            },
        },
    }
}
