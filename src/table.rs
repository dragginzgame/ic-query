const COLUMN_GAP: &str = "   ";

///
/// ColumnAlign
///
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum ColumnAlign {
    Left,
    Right,
}

#[must_use]
pub fn render_table<const N: usize>(
    headers: &[&str; N],
    rows: &[[String; N]],
    alignments: &[ColumnAlign; N],
) -> String {
    let widths = table_widths(headers, rows);
    let mut lines = Vec::with_capacity(rows.len() + 2);
    lines.push(render_table_row(headers, &widths, alignments));
    lines.push(render_separator(&widths));
    lines.extend(
        rows.iter()
            .map(|row| render_table_row(row, &widths, alignments)),
    );
    lines.join("\n")
}

#[must_use]
pub fn table_widths<const N: usize>(headers: &[&str; N], rows: &[[String; N]]) -> [usize; N] {
    let mut widths = headers.map(str::chars).map(Iterator::count);

    for row in rows {
        for (index, cell) in row.iter().enumerate() {
            widths[index] = widths[index].max(cell.chars().count());
        }
    }

    widths
}

#[must_use]
pub fn render_table_row<const N: usize>(
    row: &[impl AsRef<str>],
    widths: &[usize; N],
    alignments: &[ColumnAlign; N],
) -> String {
    widths
        .iter()
        .zip(alignments)
        .enumerate()
        .map(|(index, (width, alignment))| {
            let value = row.get(index).map_or("", AsRef::as_ref);
            pad_cell(value, *width, *alignment)
        })
        .collect::<Vec<_>>()
        .join(COLUMN_GAP)
        .trim_end()
        .to_string()
}

#[must_use]
pub fn render_separator<const N: usize>(widths: &[usize; N]) -> String {
    widths
        .iter()
        .map(|width| "-".repeat(*width))
        .collect::<Vec<_>>()
        .join(COLUMN_GAP)
}

fn pad_cell(value: &str, width: usize, alignment: ColumnAlign) -> String {
    let padding = " ".repeat(width.saturating_sub(value.chars().count()));
    match alignment {
        ColumnAlign::Left => format!("{value}{padding}"),
        ColumnAlign::Right => format!("{padding}{value}"),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn render_table_handles_long_left_aligned_cells() {
        let rows = [[
            "ICRC-1".to_string(),
            "https://github.com/dfinity/ICRC-1?with=a-long-token-metadata-url".to_string(),
        ]];

        let table = render_table(
            &["STANDARD", "URL"],
            &rows,
            &[ColumnAlign::Left, ColumnAlign::Left],
        );

        assert!(table.contains("ICRC-1"));
        assert!(table.contains("a-long-token-metadata-url"));
    }

    #[test]
    fn render_table_right_aligns_cells() {
        let rows = [["1".to_string(), "Dragginz".to_string()]];

        let table = render_table(
            &["ID", "NAME"],
            &rows,
            &[ColumnAlign::Right, ColumnAlign::Left],
        );

        assert!(table.contains("ID   NAME"));
        assert!(table.contains(" 1   Dragginz"));
    }
}
