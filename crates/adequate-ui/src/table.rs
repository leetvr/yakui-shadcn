//! Tables keep row identity separate from sorting, filtering and pagination.
use crate::{Button, Color, Variant, column, row, surface, theme, width};
use std::collections::BTreeSet;
/// A column's heading, fixed logical width and sort contract.
#[derive(Debug, Clone)]
pub struct TableColumn {
    pub title: String,
    pub width: f32,
    pub numeric: bool,
}
impl TableColumn {
    pub fn new(title: impl Into<String>, width: f32) -> Self {
        Self {
            title: title.into(),
            width,
            numeric: false,
        }
    }
    pub fn numeric(mut self) -> Self {
        self.numeric = true;
        self
    }
}
/// Rows have stable IDs so selected records survive reorder and pagination.
#[derive(Debug, Clone)]
pub struct TableRow {
    pub id: u64,
    pub cells: Vec<String>,
}
impl TableRow {
    pub fn new(id: u64, cells: impl IntoIterator<Item = impl Into<String>>) -> Self {
        Self {
            id,
            cells: cells.into_iter().map(Into::into).collect(),
        }
    }
}
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SortDirection {
    Ascending,
    Descending,
}
/// Application-owned table view state. The input records remain immutable.
#[derive(Debug, Clone)]
pub struct DataTableState {
    pub filter: String,
    pub sort: Option<(usize, SortDirection)>,
    pub page: usize,
    pub page_size: usize,
    pub selected: BTreeSet<u64>,
    pub hidden_columns: BTreeSet<usize>,
}
impl Default for DataTableState {
    fn default() -> Self {
        Self {
            filter: String::new(),
            sort: None,
            page: 0,
            page_size: 10,
            selected: BTreeSet::new(),
            hidden_columns: BTreeSet::new(),
        }
    }
}
impl DataTableState {
    pub fn sort_by(&mut self, column: usize) {
        self.sort = Some((
            column,
            match self.sort {
                Some((old, SortDirection::Ascending)) if old == column => SortDirection::Descending,
                _ => SortDirection::Ascending,
            },
        ));
    }
    /// Return record indices in view order; IDs and the source data do not change.
    pub fn view(&self, columns: &[TableColumn], rows: &[TableRow]) -> Vec<usize> {
        let query = self.filter.to_lowercase();
        let mut view: Vec<_> = rows
            .iter()
            .enumerate()
            .filter(|(_, r)| {
                query.is_empty() || r.cells.iter().any(|c| c.to_lowercase().contains(&query))
            })
            .map(|(i, _)| i)
            .collect();
        if let Some((col, direction)) = self.sort {
            if let Some(column) = columns.get(col) {
                view.sort_by(|a, b| {
                    let a = rows[*a].cells.get(col).map_or("", String::as_str);
                    let b = rows[*b].cells.get(col).map_or("", String::as_str);
                    let ordering = if column.numeric {
                        let numeric = |v: &str| {
                            v.replace(['$', ','], "")
                                .parse::<f64>()
                                .unwrap_or(f64::NEG_INFINITY)
                        };
                        numeric(a).total_cmp(&numeric(b))
                    } else {
                        a.to_lowercase().cmp(&b.to_lowercase())
                    };
                    if direction == SortDirection::Descending {
                        ordering.reverse()
                    } else {
                        ordering
                    }
                });
            }
        }
        view
    }
}
#[track_caller]
fn records(
    columns: &[TableColumn],
    rows: &[TableRow],
    view: &[usize],
    state: Option<&mut DataTableState>,
) {
    let t = theme();
    let total_width: f32 = columns
        .iter()
        .enumerate()
        .filter(|(i, _)| state.as_ref().is_none_or(|s| !s.hidden_columns.contains(i)))
        .map(|(_, c)| c.width)
        .sum();
    let mut state = state;
    width(total_width + if state.is_some() { 32. } else { 0. }, || {
        column(0., || {
            row(0., || {
                if let Some(state) = state.as_deref_mut() {
                    let all = !view.is_empty()
                        && view.iter().all(|i| state.selected.contains(&rows[*i].id));
                    let mut select_all = all;
                    width(32., || {
                        if crate::checkbox("", &mut select_all).clicked {
                            for i in view {
                                if select_all {
                                    state.selected.insert(rows[*i].id);
                                } else {
                                    state.selected.remove(&rows[*i].id);
                                }
                            }
                        }
                    });
                }
                for (i, col) in columns.iter().enumerate() {
                    if state
                        .as_ref()
                        .is_some_and(|s| s.hidden_columns.contains(&i))
                    {
                        continue;
                    }
                    width(col.width, || {
                        surface(Color::CLEAR, None, 0., 8., || {
                            if let Some(s) = state.as_deref_mut() {
                                if Button::new(col.title.clone())
                                    .variant(Variant::Ghost)
                                    .show()
                                    .clicked
                                {
                                    s.sort_by(i);
                                }
                            } else {
                                crate::label(col.title.clone());
                            }
                        })
                    });
                }
            });
            crate::separator(total_width + if state.is_some() { 32. } else { 0. });
            for &index in view {
                let record = &rows[index];
                let is_selected = state
                    .as_ref()
                    .is_some_and(|s| s.selected.contains(&record.id));
                surface(
                    if is_selected { t.muted } else { Color::CLEAR },
                    None,
                    0.,
                    0.,
                    || {
                        row(0., || {
                            if let Some(s) = state.as_deref_mut() {
                                let mut selected = s.selected.contains(&record.id);
                                width(32., || {
                                    if crate::checkbox("", &mut selected).clicked {
                                        if selected {
                                            s.selected.insert(record.id);
                                        } else {
                                            s.selected.remove(&record.id);
                                        }
                                    }
                                });
                            }
                            for (i, col) in columns.iter().enumerate() {
                                if state
                                    .as_ref()
                                    .is_some_and(|s| s.hidden_columns.contains(&i))
                                {
                                    continue;
                                }
                                width(col.width, || {
                                    surface(Color::CLEAR, None, 0., 8., || {
                                        crate::text(
                                            record.cells.get(i).cloned().unwrap_or_default(),
                                        )
                                    })
                                });
                            }
                        })
                    },
                );
                crate::separator(total_width + if state.is_some() { 32. } else { 0. });
            }
            if view.is_empty() {
                surface(Color::CLEAR, None, 0., 24., || crate::muted("No results."));
            }
        })
    });
}
/// A presentation table. The application supplies headers and stable row records.
#[track_caller]
pub fn table(columns: &[TableColumn], rows: &[TableRow], caption: Option<&str>) {
    column(16., || {
        records(columns, rows, &(0..rows.len()).collect::<Vec<_>>(), None);
        if let Some(caption) = caption {
            crate::muted(caption);
        }
    });
}
/// Filter, sort, checkbox selection, column visibility and paginated records.
#[track_caller]
pub fn data_table(columns: &[TableColumn], rows: &[TableRow], state: &mut DataTableState) {
    state.page_size = state.page_size.max(1);
    let filtered = state.view(columns, rows);
    let pages = filtered.len().div_ceil(state.page_size);
    state.page = state.page.min(pages.saturating_sub(1));
    let start = state.page * state.page_size;
    let end = (start + state.page_size).min(filtered.len());
    column(16., || {
        row(12., || {
            width(280., || {
                if crate::input(&mut state.filter, "Filter records...") {
                    state.page = 0;
                }
            });
        });
        // Visibility controls are explicit and use the same column indices as sorting.
        row(8., || {
            for (i, column) in columns.iter().enumerate() {
                let mut shown = !state.hidden_columns.contains(&i);
                if crate::checkbox(&column.title, &mut shown).clicked {
                    if shown {
                        state.hidden_columns.remove(&i);
                    } else {
                        state.hidden_columns.insert(i);
                    }
                }
            }
        });
        records(columns, rows, &filtered[start..end], Some(state));
        row(16., || {
            crate::muted(format!(
                "{} of {} row(s) selected.",
                state.selected.len(),
                rows.len()
            ));
            crate::pagination(&mut state.page, pages);
        });
    });
}
#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn filter_sort_and_selection_preserve_identity() {
        let columns = [
            TableColumn::new("Email", 100.),
            TableColumn::new("Amount", 100.).numeric(),
        ];
        let rows = [
            TableRow::new(7, ["b@example.com", "$9.00"]),
            TableRow::new(8, ["a@example.com", "$100.00"]),
            TableRow::new(9, ["c@other.net", "$2.00"]),
        ];
        let mut state = DataTableState::default();
        state.selected.insert(7);
        state.filter = "example".into();
        state.sort_by(1);
        assert_eq!(state.view(&columns, &rows), [0, 1]);
        state.sort_by(1);
        assert_eq!(state.view(&columns, &rows), [1, 0]);
        assert!(state.selected.contains(&7));
    }
}
