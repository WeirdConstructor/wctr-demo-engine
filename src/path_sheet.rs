use chrono::DateTime;
use chrono::offset::Utc;
use crate::fm_page::*;
use std::fs;

#[derive(Debug)]
pub enum FMError {
    IOError(std::io::Error),
}

impl std::convert::From<std::io::Error> for FMError {
    fn from(error: std::io::Error) -> Self {
        FMError::IOError(error)
    }
}

#[derive(Debug, PartialEq, Clone)]
pub enum PathRecordType {
    File,
    Dir,
    SymLink,
}

#[derive(Debug, PartialEq, Clone)]
pub struct PathRecord {
    pub path:       std::path::PathBuf,
    pub size:       u64,
    pub mtime:      std::time::SystemTime,
    pub path_type:  PathRecordType,
}

#[derive(Debug, PartialEq, Clone)]
pub struct IndexRefs {
    cursor: PathRecord,
    selection: Vec<PathRecord>,
    highlight: Vec<PathRecord>,
}

pub struct PathSheet {
    pub base:               std::path::PathBuf,
    pub paths:              std::vec::Vec<PathRecord>,
    pub paths_dirty:        bool,
    pub state_dirty:        bool,
    pub cursor_idx:         usize,
    pub scroll_offset:      usize,
    pub sort_col_idx:       usize,
    pub sort_col_dir:       bool,
    pub selection:          std::collections::HashSet<usize>,
    pub highlight:          std::collections::HashSet<usize>,
    pub render_feedback:    RenderFeedback,
}

const SCROLL_PADDING : usize = 5;

fn read_path_records(path: &std::path::Path) -> Result<Vec<PathRecord>, FMError>
{
    let mut sheet_paths = Vec::new();

    for e in fs::read_dir(path)? {
        let entry = e?;
        let path  = entry.path();
        let md    = path.symlink_metadata()?;
        let ft    = md.file_type();

        let pr = PathRecord {
            path,
            size:  md.len(),
            mtime: md.modified()?,
            path_type: if ft.is_symlink() {
                PathRecordType::SymLink
            } else if ft.is_dir() {
                PathRecordType::Dir
            } else {
                PathRecordType::File
            },
        };

        sheet_paths.push(pr);
    }

    Ok(sheet_paths)
}

impl PathSheet {
    pub fn get_index_refs(&self) -> IndexRefs {
        IndexRefs {
            cursor:
                self.paths[self.cursor_idx].clone(),
            selection:
                self.selection.iter().map(|i| self.paths[*i].clone()).collect(),
            highlight:
                self.highlight.iter().map(|i| self.paths[*i].clone()).collect(),
        }
    }

    pub fn select_index_refs(&mut self, idx_refs: &IndexRefs)
    {
        let new_cursor_record =
            self.paths.iter()
                .enumerate()
                .find(|(_, pr)| pr.path == idx_refs.cursor.path);
        match new_cursor_record {
            Some((i, _)) => self.cursor_idx = i,
            None         => self.cursor_idx = 0,
        }
    }

    pub fn update(&mut self) -> Result<(), FMError> {
        let old_refs = self.get_index_refs();
        std::mem::replace(&mut self.paths, read_path_records(&self.base)?);

//        for (idx, pr) in old_paths.iter().enumerate() {
//            if pr.path == cursor_record.path {
//                self.cursor_idx = idx;
//                break;
//            }
//        }
//        let selected_records    : Vec<PathRecord> =
//            self.selection.iter().map(|i| old_paths[*i]).collect();
//        let highlighted_records : Vec<PathRecord> =
//            self.highlight.iter().map(|i| old_paths[*i]).collect();

        self.paths_dirty = true;
        self.state_dirty = true;

        self.sort_by_column(self.sort_col_idx, self.sort_col_dir);

        self.select_index_refs(&old_refs);

        Ok(())
    }

    pub fn read(path: &std::path::Path) -> Result<PathSheet, FMError> {
        let mut ps = PathSheet {
            base:           path.to_path_buf(),
            paths:          read_path_records(path)?,
            cursor_idx:     0,
            scroll_offset:  0,
            sort_col_idx:   0,
            sort_col_dir:   true,
            render_feedback: RenderFeedback {
                recent_line_count: 0,
                row_offset:        0,
                start_rows:        (0, 0),
                row_height:        0,
                end_rows:          (0, 0),
            },
            selection:      std::collections::HashSet::new(),
            highlight:      std::collections::HashSet::new(),
            paths_dirty:    false,
            state_dirty:    false,
        };

        ps.sort_by_column(0, true);

        Ok(ps)
    }

    fn toggle_sort_by_column(&mut self, col_idx: usize) {
        if self.sort_col_idx == col_idx {
            self.sort_col_dir = !self.sort_col_dir;
        } else {
            self.sort_col_idx = col_idx;
            self.sort_col_dir = true;
        }

        self.sort_by_column(col_idx, self.sort_col_dir);
    }

    fn sort_by_column(&mut self, col_idx: usize, ascending: bool) {
        let old_refs = self.get_index_refs();

        if col_idx == 0 {
            self.paths.sort_by(|a, b| {
                let s1 = String::from(
                    a.path.file_name()
                    .unwrap_or(std::ffi::OsStr::new(""))
                    .to_string_lossy()).to_lowercase();
                let s2 = String::from(
                    b.path.file_name()
                    .unwrap_or(std::ffi::OsStr::new(""))
                    .to_string_lossy()).to_lowercase();

                if let PathRecordType::Dir = a.path_type {
                    if let PathRecordType::Dir = b.path_type {
                        s1.partial_cmp(&s2).unwrap()
                    } else {
                        std::cmp::Ordering::Less
                    }
                } else {
                    if let PathRecordType::Dir = b.path_type {
                        std::cmp::Ordering::Greater
                    } else {
                        s1.partial_cmp(&s2).unwrap()
                    }
                }
            });
        } else if col_idx == 1 {
            self.paths.sort_by(|a, b| a.mtime.partial_cmp(&b.mtime).unwrap());
        } else if col_idx == 2 {
            self.paths.sort_by(|a, b| a.size.partial_cmp(&b.size).unwrap());
        }

        if !ascending {
            self.paths = self.paths.iter().map(|pr| pr.clone()).rev().collect();
        }

        self.select_index_refs(&old_refs);

        self.paths_dirty = true;
    }
}

impl FmPage for PathSheet {
    fn len(&self) -> usize { self.paths.len() }
    fn get_scroll_offs(&self) -> usize { self.scroll_offset }
    fn is_cursor_idx(&self, idx: usize) -> bool { self.cursor_idx == idx }
    fn is_selected(&self, idx: usize) -> bool { self.selection.get(&idx).is_some() }
    fn is_highlighted(&self, idx: usize) -> bool { self.highlight.get(&idx).is_some() }
    fn needs_repage(&self) -> bool { self.paths_dirty }
    fn needs_redraw(&self) -> bool { self.state_dirty }

    fn set_render_feedback(&mut self, fb: RenderFeedback) {
        self.render_feedback = fb;
    }

    fn do_control(&mut self, ctrl: PageControl) {
        match ctrl {
            PageControl::ToggleSortBy(col) => {
                match col {
                    SortColumn::Name  => self.toggle_sort_by_column(0),
                    SortColumn::MTime => self.toggle_sort_by_column(1),
                    SortColumn::Size  => self.toggle_sort_by_column(2),
                }
            },
            PageControl::Update => {
                self.update().expect("Reading paths on update should work");
            },
            PageControl::CursorDown => {
                self.cursor_idx += 1;
            },
            PageControl::CursorUp => {
                if self.cursor_idx > 0 {
                    self.cursor_idx -= 1;
                }
            },
            PageControl::Click((x, y)) => {
                println!("CLICK {}, {} | {:?}", x, y, self.render_feedback);
                let x1 = self.render_feedback.start_rows.0;
                let x2 = self.render_feedback.end_rows.0;
                let y1 = self.render_feedback.start_rows.1;
                let y2 = self.render_feedback.end_rows.1;

                if !(x >= x1 && x <= x2 && y >= y1 && y <= y2) {
                    return;
                }

                let y = y - y1;
                let row = y / self.render_feedback.row_height;
                self.cursor_idx = self.render_feedback.row_offset + row as usize;
            },
            PageControl::Scroll(amount) => {
                println!("SCROLL {}", amount);
                let amount = amount * SCROLL_PADDING as i32;
                if amount < 0 && self.scroll_offset < (-amount) as usize {
                    self.scroll_offset = 0;

                } else if amount < 0 {
                    self.scroll_offset -= (-amount) as usize;

                } else {
                    self.scroll_offset += amount as usize;
                }

                if self.len() <= self.render_feedback.recent_line_count {
                    self.scroll_offset = 0;
                } else {
                    if self.scroll_offset > (self.len() - self.render_feedback.recent_line_count) {
                        self.scroll_offset = self.len() - self.render_feedback.recent_line_count;
                    }
                }

                return;
            },
            _ => {},
        }

        println!("CURSOR CTRL {} len:{}, offs:{} disp:{}",
                 self.cursor_idx,
                 self.len(),
                 self.scroll_offset,
                 self.render_feedback.recent_line_count);

        if self.cursor_idx >= self.len() {
            self.cursor_idx = if self.len() > 0 { self.len() - 1 } else { 0 };
        }

        let recent_linecnt = self.render_feedback.recent_line_count;

        if recent_linecnt <= 2 * SCROLL_PADDING {
            if self.cursor_idx > 0 {
                self.scroll_offset = self.cursor_idx - 1;
            } else {
                self.scroll_offset = self.cursor_idx;
            }
        } else {
            if self.cursor_idx < (self.scroll_offset + SCROLL_PADDING) {
                let diff = (self.scroll_offset + SCROLL_PADDING) - self.cursor_idx;
                if self.scroll_offset > diff {
                    self.scroll_offset -= diff;
                } else {
                    self.scroll_offset = 0;
                }

            } else if (self.cursor_idx + SCROLL_PADDING + 1) > (self.scroll_offset + recent_linecnt) {
                self.scroll_offset += (self.cursor_idx + SCROLL_PADDING + 1) - (self.scroll_offset + recent_linecnt);
            }

            if (self.scroll_offset + recent_linecnt) > self.len() {
                if self.len() < recent_linecnt {
                    self.scroll_offset = 0;
                } else {
                    self.scroll_offset = self.len() - recent_linecnt;
                }
            }
        }

        println!("END CURSOR CTRL {} len:{}, offs:{} disp:{}", self.cursor_idx, self.len(), self.scroll_offset, recent_linecnt);
    }

    fn as_draw_page(&self) -> Table {
        Table {
            title: String::from(self.base.to_string_lossy()),
            row_gap: 2,
            col_gap: 4,
            columns: vec![
                Column {
                    head: String::from("name"),
                    size: ColumnSizing::ExpandFract(1),
                    calc_size: None,
                    rows: self.paths.iter().map(|p| {
                        let mut path_postfix = String::from("");
                        if let PathRecordType::Dir = p.path_type {
                            path_postfix = std::path::MAIN_SEPARATOR.to_string();
                        };

                        StyleString {
                            text: String::from(p.path.file_name()
                                                .unwrap_or(std::ffi::OsStr::new(""))
                                                .to_string_lossy()) + &path_postfix,
                            style: match p.path_type {
                                PathRecordType::File    => Style::File,
                                PathRecordType::Dir     => Style::Dir,
                                PathRecordType::SymLink => Style::Special,
                            }
                        }
                    }).collect(),
                },
                Column {
                    head: String::from("time"),
                    size: ColumnSizing::TextWidth(String::from("MMMM-MM-MM MM:MM:MM")),
                    calc_size: None,
                    rows: self.paths.iter().map(|p| {
                        let dt : DateTime<Utc> = p.mtime.into();
                        StyleString { text: format!("{}", dt.format("%Y-%m-%d %H:%M:%S")), style: Style::Default }
                    }).collect(),
                },
                Column {
                    head: String::from("size"),
                    size: ColumnSizing::TextWidth(String::from("MMMMMM")),
                    calc_size: None,
                    rows: self.paths.iter().map(|p| {
                        StyleString { text: format!("{}", p.size), style: Style::Default }
                    }).collect(),
                },
            ],
        }
    }
}

