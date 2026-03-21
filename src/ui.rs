use comfy_table::{Attribute, Cell, Color, Table, presets::UTF8_FULL};
use console::style;
use indicatif::{ProgressBar, ProgressStyle};
use std::time::Duration;

use crate::recipe::RecipeSummary;

pub fn spinner(message: &str) -> ProgressBar {
    let spinner = ProgressBar::new_spinner();
    spinner.set_style(
        ProgressStyle::with_template("{spinner:.cyan} {msg}")
            .expect("spinner template should be valid")
            .tick_chars("⠋⠙⠹⠸⠼⠴⠦⠧⠇⠏"),
    );
    spinner.set_message(message.to_string());
    spinner.enable_steady_tick(Duration::from_millis(80));
    spinner
}

pub fn info(message: &str) {
    println!("{} {}", style("info").blue().bold(), message);
}

pub fn success(message: &str) {
    println!(
        "{} {}",
        style("success").green().bold(),
        style(message).bold()
    );
}

pub fn section(title: &str) {
    println!("{}", style(title).bold());
}

pub struct KeyValueRow {
    pub key: String,
    pub value: String,
    pub value_color: Option<Color>,
}

pub fn key_value_table(rows: &[KeyValueRow]) {
    let mut table = Table::new();
    table.load_preset(UTF8_FULL);
    table.set_header(vec![
        Cell::new("Field").add_attribute(Attribute::Bold),
        Cell::new("Value").add_attribute(Attribute::Bold),
    ]);

    for row in rows {
        let mut value_cell = Cell::new(row.value.as_str());
        if let Some(color) = row.value_color {
            value_cell = value_cell.fg(color);
        }

        table.add_row(vec![Cell::new(&row.key).fg(Color::Cyan), value_cell]);
    }

    println!("{table}");
}

pub fn numbered_steps(steps: &[String]) {
    for (index, step) in steps.iter().enumerate() {
        println!("{}. {}", index + 1, step);
    }
}

pub fn code(text: impl AsRef<str>) -> String {
    let text = text.as_ref();
    style(text).cyan().bold().to_string()
}

pub fn reference(kind: &str, name: impl AsRef<str>) -> String {
    let name = name.as_ref();
    format!(
        "{} {}",
        style(kind).dim().italic(),
        style(name).cyan().bold()
    )
}

pub fn display_recipe_table(recipes: &[RecipeSummary]) {
    let mut table = Table::new();
    table.load_preset(UTF8_FULL);
    table.set_header(vec![
        Cell::new("Recipe").add_attribute(Attribute::Bold),
        Cell::new("Title").add_attribute(Attribute::Bold),
        Cell::new("Difficulty").add_attribute(Attribute::Bold),
        Cell::new("Path").add_attribute(Attribute::Bold),
    ]);

    for recipe in recipes {
        table.add_row(vec![
            Cell::new(&recipe.id).fg(Color::Cyan),
            Cell::new(&recipe.title),
            Cell::new(&recipe.difficulty).fg(difficulty_color(&recipe.difficulty)),
            Cell::new(recipe.path.display().to_string()).fg(Color::DarkGrey),
        ]);
    }

    println!("{}", style("Available recipes").bold());
    println!();
    println!("{table}");
}

pub fn display_doctor_table(rows: &[DoctorRow]) {
    let mut table = Table::new();
    table.load_preset(UTF8_FULL);
    table.set_header(vec![
        Cell::new("Tool").add_attribute(Attribute::Bold),
        Cell::new("Status").add_attribute(Attribute::Bold),
        Cell::new("When").add_attribute(Attribute::Bold),
        Cell::new("Location").add_attribute(Attribute::Bold),
        Cell::new("Notes").add_attribute(Attribute::Bold),
    ]);

    for row in rows {
        table.add_row(vec![
            Cell::new(&row.tool).fg(Color::Cyan),
            Cell::new(&row.status).fg(row.status_color),
            Cell::new(&row.when).fg(row.when_color),
            Cell::new(row.location.as_deref().unwrap_or("-")),
            Cell::new(&row.notes).fg(Color::DarkGrey),
        ]);
    }

    println!("{table}");
}

pub struct DoctorRow {
    pub tool: String,
    pub status: String,
    pub status_color: Color,
    pub when: String,
    pub when_color: Color,
    pub location: Option<String>,
    pub notes: String,
}

fn difficulty_color(difficulty: &str) -> Color {
    match difficulty {
        "easy" => Color::Green,
        "medium" => Color::Yellow,
        "hard" => Color::Red,
        "expert" => Color::Magenta,
        _ => Color::White,
    }
}
