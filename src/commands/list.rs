use anyhow::Result;
use std::path::Path;

use crate::recipe;
use crate::ui;

pub fn run(primer_root: &Path) -> Result<()> {
    let spinner = ui::spinner("Scanning Primer recipes...");
    let recipes = recipe::discover(primer_root)?;
    spinner.finish_and_clear();

    if recipes.is_empty() {
        ui::info("No recipes found.");
        return Ok(());
    }

    ui::display_recipe_table(&recipes);

    Ok(())
}
