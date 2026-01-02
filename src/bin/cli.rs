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
        Commands::Theme { name } => match ThemeService::set_theme(&name) {
            Ok(()) => println!("The theme was set successfully"),
            Err(e) => eprintln!("Error setting theme: {e}"),
        },
        Commands::Wallpaper { action } => match action {
            WallpaperAction::Reload => match ThemeService::change_wallpaper() {
                Ok(()) => println!("The wallpaper was reloaded"),
                Err(e) => eprintln!("Error reloading wallpaper: {e}"),
            },
        },
    }
}
