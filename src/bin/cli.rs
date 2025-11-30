use clap::{Parser, Subcommand};

#[derive(Parser)]
#[command(name = "norlyk", about = "Norlyk settings manager")]
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
        Commands::Theme { name } => match theme_picker::services::theme::set_theme(&name) {
            Ok(_) => println!("The theme was set successfully"),
            Err(e) => eprintln!("Error setting theme: {e}"),
        },
        Commands::Wallpaper { action } => match action {
            WallpaperAction::Reload => match theme_picker::services::theme::change_wallpaper() {
                Ok(_) => println!("The wallpaper was reloaded"),
                Err(e) => eprintln!("Error reloading wallpaper: {e}"),
            },
        },
    }
}
