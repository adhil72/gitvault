//! Human-readable rendering helpers.

/// Renders rows as a left-aligned text table with a header.
pub fn table(headers: &[&str], rows: &[Vec<String>]) -> String {
    let mut widths: Vec<usize> = headers.iter().map(|h| h.chars().count()).collect();
    for row in rows {
        for (i, cell) in row.iter().enumerate() {
            if let Some(width) = widths.get_mut(i) {
                *width = (*width).max(cell.chars().count());
            }
        }
    }
    let line = |cells: Vec<&str>| -> String {
        cells
            .iter()
            .enumerate()
            .map(|(i, c)| format!("{c:<width$}", width = widths.get(i).copied().unwrap_or(0)))
            .collect::<Vec<_>>()
            .join("  ")
            .trim_end()
            .to_owned()
    };
    let mut out = vec![line(headers.to_vec())];
    out.extend(
        rows.iter()
            .map(|row| line(row.iter().map(String::as_str).collect())),
    );
    out.join("\n")
}

#[cfg(test)]
mod tests {
    use super::table;

    #[test]
    fn aligns_columns() {
        let text = table(
            &["NAME", "N"],
            &[
                vec!["alpha".into(), "1".into()],
                vec!["b".into(), "22".into()],
            ],
        );
        assert_eq!(text, "NAME   N\nalpha  1\nb      22");
    }
}
