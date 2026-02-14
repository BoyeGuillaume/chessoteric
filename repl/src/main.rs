pub mod app;
pub mod board;
pub mod skin;

fn main() -> color_eyre::Result<()> {
    color_eyre::install().unwrap();
    ratatui::run(app::app)?;
    Ok(())
}
