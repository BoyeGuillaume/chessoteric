pub mod app;
pub mod board;
pub mod skin;

fn main() -> color_eyre::Result<()> {
    color_eyre::install().unwrap();
    let output = ratatui::run(app::app)?;
    println!("{}", output);
    Ok(())
}
