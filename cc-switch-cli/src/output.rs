use comfy_table::{presets, Table};

pub fn create_table(headers: Vec<&str>) -> Table {
    let mut table = Table::new();
    table.load_preset(presets::UTF8_FULL);
    table.set_header(headers);
    table
}
