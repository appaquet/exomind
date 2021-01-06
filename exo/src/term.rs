use std::fmt::{Debug, Display};

use console::style;

pub fn print_spacer() {
    println!();
}

pub fn print_step<S: Display>(text: S) {
    println!("\n{} {}", style("✔").green().bold(), text);
}

pub fn print_action<S: Display>(text: S) {
    println!("  - {}", text);
}

pub fn print_info<S: Display>(text: S) {
    println!("\n{} {}", style("i").yellow().bold(), text);
}

pub fn print_success<S: Display>(text: S) {
    println!("\n{} {}", style("✔").bold().green(), text);
}

pub fn print_error<S: Display>(text: S) {
    println!("\n{} {}", style("!").bold().red(), text);
}

pub fn style_value<S: Debug>(value: S) -> String {
    format!("{:?}", style(value).green(),)
}

pub fn style_emphasis<S: Display>(value: S) -> String {
    format!("{}", style(value).bold(),)
}

pub fn style_err<S: Debug>(value: S) -> String {
    format!("{:?}", style(value).red(),)
}

pub fn print_table(columns: Vec<String>, rows: Vec<Vec<String>>) {
    let mut col_size = Vec::new();
    for col in &columns {
        col_size.push(col.len());
    }
    for row in &rows {
        for (i, col) in row.iter().enumerate() {
            col_size[i] = col_size[i].max(col.len());
        }
    }

    // Print columns
    for (i, col) in columns.iter().enumerate() {
        let expected_size = col_size[i];
        let len = col.len();
        let spaces = " ".repeat(expected_size - len);
        print!("{}{} {} ", style(col).bold(), spaces, style("|").dim());
    }
    println!();

    // Print separator
    for col in &col_size {
        print!("{}", style(format!("{}-{} ", "-".repeat(*col), "|")).dim());
    }
    println!();

    // Print rows
    for row in &rows {
        for (i, col) in row.iter().enumerate() {
            let expected_size = col_size[i];
            let len = col.len();
            let spaces = " ".repeat(expected_size - len);
            print!("{}{} {} ", col, spaces, style("|").dim());
        }
        println!();
    }
}
