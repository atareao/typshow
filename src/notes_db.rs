use rusqlite::{Connection, params};
use std::path::PathBuf;

pub struct NoteRecord {
    pub page_number: usize,
    pub heading: Option<String>,
    pub content: String,
}

pub struct TypSegment {
    #[allow(dead_code)]
    pub start_line: usize,
    #[allow(dead_code)]
    pub end_line: usize,
    pub heading: Option<String>,
    pub content: String,
    pub heading_level: u32,
}

pub struct NotesDb {
    conn: Connection,
    path: PathBuf,
}

fn parse_typ_title(source: &str) -> Option<String> {
    let idx = source.find("title:")?;
    let after = source[idx + 6..].trim_start();
    let rest = after.strip_prefix('"')?;
    let mut escaped = false;
    let mut end = 0;
    for (i, ch) in rest.char_indices() {
        if escaped { escaped = false; continue; }
        if ch == '\\' { escaped = true; continue; }
        if ch == '"' { end = i; break; }
    }
    if end > 0 { Some(rest[..end].to_string()) } else { None }
}

impl NotesDb {
    pub fn open(typ_path: &str) -> Result<Self, Box<dyn std::error::Error>> {
        let p = std::path::Path::new(typ_path);
        let db_path = p.with_extension("typshow.db");

        let conn = Connection::open(&db_path)?;
        conn.execute_batch("PRAGMA journal_mode=WAL")?;

        let db = Self { conn, path: db_path };
        db.init_schema()?;
        Ok(db)
    }

    fn init_schema(&self) -> Result<(), Box<dyn std::error::Error>> {
        self.conn.execute_batch(
            "CREATE TABLE IF NOT EXISTS notes (
                page_number INTEGER PRIMARY KEY,
                heading     TEXT,
                content     TEXT NOT NULL DEFAULT '',
                updated_at  TEXT NOT NULL DEFAULT (datetime('now'))
            );"
        )?;
        Ok(())
    }

    pub fn path(&self) -> &std::path::Path {
        &self.path
    }

    pub fn has_data(&self) -> Result<bool, Box<dyn std::error::Error>> {
        let count: i64 = self.conn.query_row(
            "SELECT COUNT(*) FROM notes", [], |r| r.get(0)
        )?;
        Ok(count > 0)
    }

    pub fn load(&self, page: usize) -> Result<NoteRecord, Box<dyn std::error::Error>> {
        let mut stmt = self.conn.prepare(
            "SELECT page_number, heading, content FROM notes WHERE page_number = ?1"
        )?;

        let result = stmt.query_row(params![page as i64], |r| {
            Ok(NoteRecord {
                page_number: r.get::<_, i64>(0)? as usize,
                heading: r.get(1)?,
                content: r.get(2)?,
            })
        });

        match result {
            Ok(record) => Ok(record),
            Err(rusqlite::Error::QueryReturnedNoRows) => {
                Ok(NoteRecord {
                    page_number: page,
                    heading: None,
                    content: String::new(),
                })
            }
            Err(e) => Err(Box::new(e)),
        }
    }

    pub fn save(&self, page: usize, content: &str) -> Result<(), Box<dyn std::error::Error>> {
        self.conn.execute(
            "INSERT INTO notes (page_number, content, updated_at)
             VALUES (?1, ?2, datetime('now'))
             ON CONFLICT(page_number) DO UPDATE SET
                content = excluded.content,
                updated_at = datetime('now')",
            params![page as i64, content],
        )?;
        Ok(())
    }

    pub fn set_heading(&self, page: usize, heading: Option<&str>) -> Result<(), Box<dyn std::error::Error>> {
        self.conn.execute(
            "INSERT INTO notes (page_number, heading, updated_at)
             VALUES (?1, ?2, datetime('now'))
             ON CONFLICT(page_number) DO UPDATE SET
                heading = excluded.heading,
                updated_at = datetime('now')",
            params![page as i64, heading],
        )?;
        Ok(())
    }

    pub fn delete(&self, page: usize) -> Result<(), Box<dyn std::error::Error>> {
        self.conn.execute(
            "DELETE FROM notes WHERE page_number = ?1",
            params![page as i64],
        )?;
        Ok(())
    }

    pub fn insert_note(&self, page: usize) -> Result<(), Box<dyn std::error::Error>> {
        self.conn.execute(
            "INSERT OR IGNORE INTO notes (page_number, content, updated_at)
             VALUES (?1, '', datetime('now'))",
            params![page as i64],
        )?;
        Ok(())
    }

    pub fn shift_up(&self, from_page: usize) -> Result<(), Box<dyn std::error::Error>> {
        self.conn.execute(
            "UPDATE notes SET page_number = -(page_number + 1) WHERE page_number >= ?1",
            params![from_page as i64],
        )?;
        self.conn.execute(
            "UPDATE notes SET page_number = -page_number WHERE page_number < 0",
            [],
        )?;
        Ok(())
    }

    pub fn clear_all(&self) -> Result<(), Box<dyn std::error::Error>> {
        self.conn.execute("DELETE FROM notes", [])?;
        Ok(())
    }

    pub fn migrate_from_typ(
        &self,
        typ_content: &str,
        total_pages: usize,
    ) -> Result<(), Box<dyn std::error::Error>> {
        if self.has_data()? {
            return Ok(());
        }
        if total_pages == 0 {
            return Ok(());
        }

        let segments = extract_typ_segments(typ_content);

        // Page 0: presentation title
        let title = parse_typ_title(typ_content);
        self.set_heading(0, title.as_deref())?;
        self.save(0, "")?;

        if total_pages <= 1 {
            return Ok(());
        }

        if segments.is_empty() {
            for page in 1..total_pages {
                self.set_heading(page, None)?;
                self.save(page, "")?;
            }
            return Ok(());
        }

        let fixed: Vec<&TypSegment> = segments.iter().filter(|s| s.heading_level >= 1 && s.heading_level <= 2).collect();
        let flex: Vec<&TypSegment> = segments.iter().filter(|s| s.heading_level >= 3).collect();

        let mut page = 1;

        // Assign one page per = or == heading
        for seg in &fixed {
            if page >= total_pages {
                break;
            }
            self.set_heading(page, seg.heading.as_deref())?;
            self.save(page, &seg.content)?;
            page += 1;
        }

        // Distribute remaining pages among === (and higher) headings
        let flex_pages = total_pages.saturating_sub(page);

        if flex_pages > 0 && !flex.is_empty() {
            let seg_size = flex_pages / flex.len();
            let remainder = flex_pages % flex.len();

            for (i, seg) in flex.iter().enumerate() {
                let extra = if i < remainder { 1 } else { 0 };
                let seg_end = (page + seg_size + extra).min(total_pages);
                for p in page..seg_end {
                    self.set_heading(p, seg.heading.as_deref())?;
                    self.save(p, &seg.content)?;
                }
                page = seg_end;
            }
        } else if page < total_pages {
            let last_seg = segments.last();
            for p in page..total_pages {
                if let Some(seg) = last_seg {
                    self.set_heading(p, seg.heading.as_deref())?;
                    self.save(p, &seg.content)?;
                } else {
                    self.save(p, "")?;
                }
            }
        }

        Ok(())
    }
}

#[cfg(test)]
fn build_segments(
    headings: &[HeadingInfo],
    pagebreak_lines: &[usize],
    total_lines: usize,
) -> Vec<(usize, usize, Option<String>)> {
    let mut boundaries: Vec<(usize, Option<String>)> = Vec::new();
    for h in headings {
        if !boundaries.iter().any(|(l, _)| *l == h.line) {
            boundaries.push((h.line, Some(h.text.clone())));
        }
    }
    for line in pagebreak_lines {
        if !boundaries.iter().any(|(l, _)| *l == *line) {
            boundaries.push((*line, None));
        }
    }
    boundaries.sort_by_key(|(line, _)| *line);

    let mut segments: Vec<(usize, usize, Option<String>)> = Vec::new();
    let mut last_label: Option<String> = None;

    for i in 0..boundaries.len() {
        let (start, label_opt) = &boundaries[i];
        let end = boundaries.get(i + 1).map(|(l, _)| *l).unwrap_or(total_lines);

        let resolved = label_opt.clone().or_else(|| last_label.clone());
        if label_opt.is_some() {
            last_label = label_opt.clone();
        }

        segments.push((*start, end, resolved));
    }

    segments
}

pub fn parse_typ_headings(source: &str) -> Vec<HeadingInfo> {
    let mut headings = Vec::new();
    for (i, line) in source.lines().enumerate() {
        let trimmed = line.trim_start();
        let level = if trimmed.starts_with("==== ") { 4 }
        else if trimmed.starts_with("=== ") { 3 }
        else if trimmed.starts_with("== ") { 2 }
        else if trimmed.starts_with("= ") { 1 }
        else { continue };

        let text = trimmed[level..].trim().to_string();
        headings.push(HeadingInfo { level: level as u32, text, line: i });
    }
    headings
}

pub fn extract_typ_segments(source: &str) -> Vec<TypSegment> {
    let headings = parse_typ_headings(source);
    let lines: Vec<&str> = source.lines().collect();
    let total_lines = lines.len();

    let mut boundaries: Vec<(usize, Option<String>, u32)> = Vec::new();
    for h in &headings {
        if !boundaries.iter().any(|(l, _, _)| *l == h.line) {
            boundaries.push((h.line, Some(h.text.clone()), h.level));
        }
    }
    for (i, line) in lines.iter().enumerate() {
        if line.trim().starts_with("#pagebreak") {
            if !boundaries.iter().any(|(l, _, _)| *l == i) {
                boundaries.push((i, None, 0));
            }
        }
    }
    boundaries.sort_by_key(|(line, _, _)| *line);

    let mut segments = Vec::new();
    let mut last_label: Option<String> = None;

    for i in 0..boundaries.len() {
        let (start, label_opt, level) = &boundaries[i];
        let end = boundaries.get(i + 1).map(|(l, _, _)| *l).unwrap_or(total_lines);

        let resolved = label_opt.clone().or_else(|| last_label.clone());
        if label_opt.is_some() {
            last_label = label_opt.clone();
        }

        let content: String = lines[*start..end]
            .iter()
            .filter(|l| {
                let t = l.trim_start();
                !(t.starts_with("= ")
                    || t.starts_with("== ")
                    || t.starts_with("=== ")
                    || t.starts_with("==== ")
                    || t.trim_start().starts_with("#pagebreak"))
            })
            .map(|l| l.trim())
            .filter(|l| !l.is_empty())
            .collect::<Vec<&str>>()
            .join("\n");

        segments.push(TypSegment {
            start_line: *start,
            end_line: end,
            heading: resolved,
            content,
            heading_level: *level,
        });
    }

    segments
}

#[cfg(test)]
fn find_pagebreak_lines(source: &str) -> Vec<usize> {
    let mut lines = Vec::new();
    for (i, line) in source.lines().enumerate() {
        let trimmed = line.trim();
        if trimmed.starts_with("#pagebreak") {
            lines.push(i);
        }
    }
    lines
}

#[derive(Clone, Debug)]
#[allow(dead_code)]
pub struct HeadingInfo {
    pub level: u32,
    pub text: String,
    pub line: usize,
}

#[cfg(test)]
mod tests {
    use super::*;

    fn test_db() -> NotesDb {
        let conn = Connection::open_in_memory().unwrap();
        let db = NotesDb {
            conn,
            path: PathBuf::from(":memory:"),
        };
        db.init_schema().unwrap();
        db
    }

    #[test]
    fn test_save_and_load() {
        let db = test_db();
        db.save(0, "Hello world").unwrap();
        let note = db.load(0).unwrap();
        assert_eq!(note.content, "Hello world");
    }

    #[test]
    fn test_load_missing_page() {
        let db = test_db();
        let note = db.load(99).unwrap();
        assert_eq!(note.content, "");
        assert!(note.heading.is_none());
    }

    #[test]
    fn test_save_overwrites() {
        let db = test_db();
        db.save(0, "v1").unwrap();
        db.save(0, "v2").unwrap();
        let note = db.load(0).unwrap();
        assert_eq!(note.content, "v2");
    }

    #[test]
    fn test_set_heading() {
        let db = test_db();
        db.set_heading(0, Some("Intro")).unwrap();
        let note = db.load(0).unwrap();
        assert_eq!(note.heading.unwrap(), "Intro");
    }

    #[test]
    fn test_set_heading_none() {
        let db = test_db();
        db.set_heading(0, None).unwrap();
        let note = db.load(0).unwrap();
        assert!(note.heading.is_none());
    }

    #[test]
    fn test_has_data_empty() {
        let db = test_db();
        assert!(!db.has_data().unwrap());
    }

    #[test]
    fn test_has_data_with_rows() {
        let db = test_db();
        db.save(0, "data").unwrap();
        assert!(db.has_data().unwrap());
    }

    #[test]
    fn test_migrate_no_headings() {
        let db = test_db();
        db.migrate_from_typ("plain text\nno headings", 3).unwrap();
        assert_eq!(db.load(0).unwrap().heading, None);
        assert_eq!(db.load(1).unwrap().heading, None);
        assert_eq!(db.load(2).unwrap().heading, None);
    }

    #[test]
    fn test_migrate_with_headings() {
        let db = test_db();
        let typ = "= Intro\n\nSome text\n\n= Body\n\nMore text\n\n= Conclusion";
        db.migrate_from_typ(typ, 6).unwrap();
        assert!(db.has_data().unwrap());
        // No title: in typ → page 0 = None
        assert_eq!(db.load(0).unwrap().heading, None);
        // = headings: each gets 1 page
        assert_eq!(db.load(1).unwrap().heading.unwrap(), "Intro");
        assert_eq!(db.load(2).unwrap().heading.unwrap(), "Body");
        assert_eq!(db.load(3).unwrap().heading.unwrap(), "Conclusion");
        // Remaining pages inherit last heading
        assert_eq!(db.load(4).unwrap().heading.unwrap(), "Conclusion");
        assert_eq!(db.load(5).unwrap().heading.unwrap(), "Conclusion");
    }

    #[test]
    fn test_migrate_with_title() {
        let db = test_db();
        let typ = r##"#show: slides.with(
  title: "Mi Presentación",
  subtitle: "Test",
)

= Intro
Some text
= Body
More text
= End"##;
        db.migrate_from_typ(typ, 5).unwrap();
        // Page 0: parsed title
        assert_eq!(db.load(0).unwrap().heading.unwrap(), "Mi Presentación");
        // = headings: each 1 page
        assert_eq!(db.load(1).unwrap().heading.unwrap(), "Intro");
        assert_eq!(db.load(2).unwrap().heading.unwrap(), "Body");
        assert_eq!(db.load(3).unwrap().heading.unwrap(), "End");
        assert_eq!(db.load(4).unwrap().heading.unwrap(), "End");
    }

    #[test]
    fn test_migrate_idempotent() {
        let db = test_db();
        let typ = "= A\n\n= B\n\n= C";
        db.migrate_from_typ(typ, 4).unwrap();
        db.migrate_from_typ(typ, 4).unwrap();
        assert_eq!(db.load(0).unwrap().heading, None);
        assert_eq!(db.load(1).unwrap().heading.unwrap(), "A");
        assert_eq!(db.load(2).unwrap().heading.unwrap(), "B");
        assert_eq!(db.load(3).unwrap().heading.unwrap(), "C");
    }

    #[test]
    fn test_migrate_with_pagebreak() {
        let db = test_db();
        let typ = "= A\ncontent\n#pagebreak()\n= B\nmore\n#pagebreak()\n= C";
        db.migrate_from_typ(typ, 4).unwrap();
        assert!(db.has_data().unwrap());
        assert_eq!(db.load(0).unwrap().heading, None);
        assert_eq!(db.load(1).unwrap().heading.unwrap(), "A");
        assert_eq!(db.load(2).unwrap().heading.unwrap(), "B");
        assert_eq!(db.load(3).unwrap().heading.unwrap(), "C");
    }

    #[test]
    fn test_migrate_single_page() {
        let db = test_db();
        db.migrate_from_typ("= Title", 1).unwrap();
        assert_eq!(db.load(0).unwrap().heading, None);
    }

    #[test]
    fn test_migrate_mixed_levels() {
        let db = test_db();
        let typ = "= Title\n== Sec1\n=== SubA\n=== SubB\n== Sec2\n=== SubC";
        db.migrate_from_typ(typ, 10).unwrap();
        // Page 0: no title: in content
        assert_eq!(db.load(0).unwrap().heading, None);
        // Fixed (=,==): Title→1, Sec1→2, Sec2→3
        assert_eq!(db.load(1).unwrap().heading.unwrap(), "Title");
        assert_eq!(db.load(2).unwrap().heading.unwrap(), "Sec1");
        assert_eq!(db.load(3).unwrap().heading.unwrap(), "Sec2");
        // Flex (===): SubA, SubB, SubC split pages 4-9 (6 pages, 2 each)
        assert_eq!(db.load(4).unwrap().heading.unwrap(), "SubA");
        assert_eq!(db.load(5).unwrap().heading.unwrap(), "SubA");
        assert_eq!(db.load(6).unwrap().heading.unwrap(), "SubB");
        assert_eq!(db.load(7).unwrap().heading.unwrap(), "SubB");
        assert_eq!(db.load(8).unwrap().heading.unwrap(), "SubC");
        assert_eq!(db.load(9).unwrap().heading.unwrap(), "SubC");
    }

    #[test]
    fn test_parse_typ_title_basic() {
        let src = r##"#show: slides.with(
  title: "Mi Presentación",
)"##;
        assert_eq!(parse_typ_title(src).unwrap(), "Mi Presentación");
    }

    #[test]
    fn test_parse_typ_title_escaped_quote() {
        let src = r##"title: "Hello \"World\"" "##;
        assert_eq!(parse_typ_title(src).unwrap(), r##"Hello \"World\""##);
    }

    #[test]
    fn test_parse_typ_title_not_found() {
        assert!(parse_typ_title("= No title here").is_none());
    }

    #[test]
    fn test_find_pagebreak_lines() {
        let lines = find_pagebreak_lines("a\n#pagebreak()\nb\n#pagebreak");
        assert_eq!(lines, vec![1, 3]);
    }

    #[test]
    fn test_find_pagebreak_lines_none() {
        let lines = find_pagebreak_lines("no breaks here");
        assert!(lines.is_empty());
    }

    #[test]
    fn test_build_segments_empty() {
        let segs = build_segments(&[], &[], 100);
        assert!(segs.is_empty());
    }

    #[test]
    fn test_build_segments_with_headings() {
        let headings = vec![
            HeadingInfo { level: 1, text: "A".into(), line: 0 },
            HeadingInfo { level: 1, text: "B".into(), line: 10 },
        ];
        let segs = build_segments(&headings, &[], 30);
        assert_eq!(segs.len(), 2);
        assert_eq!(segs[0].2, Some("A".into()));
        assert_eq!(segs[0].1, 10);
        assert_eq!(segs[1].2, Some("B".into()));
        assert_eq!(segs[1].1, 30);
    }

    #[test]
    fn test_parse_typ_headings_levels() {
        let src = "= H1\n== H2\n=== H3\n==== H4";
        let h = parse_typ_headings(src);
        assert_eq!(h.len(), 4);
        assert_eq!(h[0].text, "H1");
        assert_eq!(h[1].text, "H2");
        assert_eq!(h[2].text, "H3");
        assert_eq!(h[3].text, "H4");
    }

    #[test]
    fn test_delete_note() {
        let db = test_db();
        db.save(0, "content").unwrap();
        assert!(db.has_data().unwrap());
        db.delete(0).unwrap();
        let note = db.load(0).unwrap();
        assert_eq!(note.content, "");
    }

    #[test]
    fn test_insert_note_existing() {
        let db = test_db();
        db.save(0, "existing").unwrap();
        db.insert_note(0).unwrap();
        let note = db.load(0).unwrap();
        assert_eq!(note.content, "existing");
    }

    #[test]
    fn test_clear_all() {
        let db = test_db();
        db.save(0, "a").unwrap();
        db.save(1, "b").unwrap();
        db.clear_all().unwrap();
        assert!(!db.has_data().unwrap());
    }

    #[test]
    fn test_extract_segments_with_headings() {
        let src = "= Intro\nsome intro text\n\n= Body\nthe body content";
        let segs = extract_typ_segments(src);
        assert_eq!(segs.len(), 2);
        assert_eq!(segs[0].heading.as_deref(), Some("Intro"));
        assert_eq!(segs[0].content, "some intro text");
        assert_eq!(segs[1].heading.as_deref(), Some("Body"));
        assert_eq!(segs[1].content, "the body content");
    }

    #[test]
    fn test_extract_segments_with_pagebreak() {
        let src = "= A\ncontent A\n#pagebreak()\n= B\ncontent B";
        let segs = extract_typ_segments(src);
        assert_eq!(segs.len(), 3);
        assert_eq!(segs[0].content, "content A");
        assert!(segs[1].content.is_empty());
        assert_eq!(segs[1].heading.as_deref(), Some("A"));
        assert_eq!(segs[2].content, "content B");
        assert_eq!(segs[2].heading.as_deref(), Some("B"));
    }

    #[test]
    fn test_extract_segments_empty() {
        let segs = extract_typ_segments("");
        assert!(segs.is_empty());
    }

    #[test]
    fn test_migrate_with_content() {
        let db = test_db();
        let typ = "= Intro\n\nSlide 1 text\n\n= Body\n\nSlide 2 text";
        db.migrate_from_typ(typ, 4).unwrap();
        assert_eq!(db.load(1).unwrap().content, "Slide 1 text");
        assert_eq!(db.load(2).unwrap().content, "Slide 2 text");
        assert_eq!(db.load(2).unwrap().heading.unwrap(), "Body");
    }

    #[test]
    fn test_migrate_uses_segment_content() {
        let db = test_db();
        let typ = "== Sec1\ndetailed\ncontent here\n== Sec2\nmore\ndetails";
        db.migrate_from_typ(typ, 3).unwrap();
        assert_eq!(db.load(1).unwrap().content, "detailed\ncontent here");
        assert_eq!(db.load(2).unwrap().content, "more\ndetails");
    }

    #[test]
    fn test_shift_up_shifts_pages() {
        let db = test_db();
        db.save(0, "page0").unwrap();
        db.save(1, "page1").unwrap();
        db.save(2, "page2").unwrap();
        db.shift_up(1).unwrap();
        assert_eq!(db.load(0).unwrap().content, "page0");
        assert_eq!(db.load(1).unwrap().content, "");
        assert_eq!(db.load(2).unwrap().content, "page1");
        assert_eq!(db.load(3).unwrap().content, "page2");
    }

    #[test]
    fn test_shift_up_from_zero() {
        let db = test_db();
        db.save(0, "a").unwrap();
        db.save(1, "b").unwrap();
        db.shift_up(0).unwrap();
        assert_eq!(db.load(0).unwrap().content, "");
        assert_eq!(db.load(1).unwrap().content, "a");
        assert_eq!(db.load(2).unwrap().content, "b");
    }

    #[test]
    fn test_shift_up_past_end() {
        let db = test_db();
        db.save(0, "a").unwrap();
        db.shift_up(5).unwrap();
        assert_eq!(db.load(0).unwrap().content, "a");
    }
}