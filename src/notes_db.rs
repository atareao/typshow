use rusqlite::{Connection, params};
use std::path::PathBuf;
use tracing::warn;

pub struct NoteRecord {
    pub page_number: usize,
    pub heading: Option<String>,
    pub content: String,
    /// Hash del Markdown embebido del que se sembró la nota (`None` si no
    /// procede de un bloque de guion embebido).
    pub source_hash: Option<String>,
    /// `true` si la nota fue editada localmente por el usuario desde el panel.
    pub edited: bool,
}

/// Centinela de apertura de un bloque de guion embebido en el `.typ`.
pub const NOTES_BLOCK_OPEN: &str = "// >>> typshow:notes";
/// Centinela de cierre de un bloque de guion embebido en el `.typ`.
pub const NOTES_BLOCK_CLOSE: &str = "// >>> typshow:notes:end";

/// Bloque de guion embebido en el `.typ`, anclado a la diapositiva cuyo heading
/// (`=` o `==`) precede inmediatamente al centinela de apertura.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct EmbeddedBlock {
    /// Texto del heading al que se ancla el bloque.
    pub heading: String,
    /// Línea (0-based) del heading ancla: la asociación bloque → página es
    /// **posicional** y no se reasocia por igualdad de texto (los títulos pueden
    /// repetirse).
    pub heading_line: usize,
    /// Contenido del bloque como Markdown (prefijo `//` de comentario eliminado).
    pub markdown: String,
    /// Índice (0-based) de la línea del centinela de apertura.
    pub start_line: usize,
    /// Índice (0-based) de la línea del centinela de cierre, inclusive.
    pub end_line: usize,
}

/// Extrae los bloques de guion embebidos del `.typ` y los ancla al último
/// heading de slide (`=`/`==`) visto antes de cada centinela de apertura.
///
/// El anclaje es **posicional**: se guarda la línea del heading, de modo que la
/// asociación a página no depende del texto del título. Un bloque sin cierre,
/// sin heading previo o anidado se descarta registrando un aviso; en el caso
/// anidado se descarta el bloque **exterior** y vale el interior. La lectura
/// nunca aborta.
pub fn parse_embedded_blocks(source: &str) -> Vec<EmbeddedBlock> {
    let lines: Vec<&str> = source.lines().collect();
    let mut blocks = Vec::new();
    let mut anchor: Option<(usize, String)> = None;
    let mut open: Option<usize> = None;

    for (i, line) in lines.iter().enumerate() {
        let trimmed = line.trim_start();

        if let Some(text) = slide_heading_text(trimmed) {
            anchor = Some((i, text));
            continue;
        }

        let marker = trimmed.trim_end();
        if marker == NOTES_BLOCK_OPEN {
            if open.is_some() {
                warn!(
                    "bloque de guion anidado en la línea {}; se descarta el bloque exterior",
                    i + 1
                );
            }
            open = Some(i);
            continue;
        }

        if marker == NOTES_BLOCK_CLOSE {
            match open.take() {
                Some(start) => {
                    let markdown = decode_block_lines(&lines[start + 1..i]);
                    match anchor.clone() {
                        Some((heading_line, heading)) => blocks.push(EmbeddedBlock {
                            heading,
                            heading_line,
                            markdown,
                            start_line: start,
                            end_line: i,
                        }),
                        None => warn!(
                            "bloque de guion sin heading previo en la línea {}; descartado",
                            start + 1
                        ),
                    }
                }
                None => warn!(
                    "centinela de cierre de guion sin apertura en la línea {}; ignorado",
                    i + 1
                ),
            }
        }
    }

    if let Some(start) = open {
        warn!(
            "bloque de guion sin cerrar en la línea {}; descartado",
            start + 1
        );
    }

    blocks
}

/// Devuelve el texto del heading si `trimmed` abre un heading de slide (`=` o `==`).
///
/// Los niveles `===` y superiores no anclan bloques de guion.
fn slide_heading_text(trimmed: &str) -> Option<String> {
    let level = if trimmed.starts_with("== ") {
        2
    } else if trimmed.starts_with("= ") {
        1
    } else {
        return None;
    };
    Some(trimmed[level..].trim().to_string())
}

/// Rangos de líneas (inclusive) ocupados por bloques embebidos, cerrados o no.
///
/// Se usan para excluir esas líneas del contenido de segmento aunque el bloque
/// se descarte por falta de anclaje: el guion nunca debe colarse como texto.
fn embedded_block_ranges(lines: &[&str]) -> Vec<(usize, usize)> {
    let mut ranges = Vec::new();
    let mut open: Option<usize> = None;
    for (i, line) in lines.iter().enumerate() {
        let marker = line.trim_start().trim_end();
        if marker == NOTES_BLOCK_OPEN {
            if open.is_none() {
                open = Some(i);
            }
        } else if marker == NOTES_BLOCK_CLOSE
            && let Some(start) = open.take()
        {
            ranges.push((start, i));
        }
    }
    if let Some(start) = open {
        ranges.push((start, lines.len().saturating_sub(1)));
    }
    ranges
}

/// Texto del marcador de apertura, sin el prefijo de comentario.
const NOTES_MARKER_OPEN: &str = ">>> typshow:notes";
/// Texto del marcador de cierre, sin el prefijo de comentario.
const NOTES_MARKER_CLOSE: &str = ">>> typshow:notes:end";

/// Decodifica las líneas de contenido de un bloque: quita el prefijo `//` y, como
/// máximo, un espacio siguiente. Las líneas vacías se conservan.
///
/// Deshace además el escape de [`escape_sentinel`].
fn decode_block_lines(lines: &[&str]) -> String {
    lines
        .iter()
        .map(|line| {
            let trimmed = line.trim_start();
            match trimmed.strip_prefix("//") {
                Some(rest) => unescape_sentinel(rest.strip_prefix(' ').unwrap_or(rest)),
                None => (*line).to_string(),
            }
        })
        .collect::<Vec<String>>()
        .join("\n")
}

/// Escapa una línea que, comentada, se confundiría con un centinela, o que ya
/// empieza por la marca de escape (para que el escape sea reversible).
///
/// Sin esto, una nota que contuviera `>>> typshow:notes` rompería el round-trip:
/// al codificarla se emitiría un centinela dentro del bloque.
fn escape_sentinel(line: &str) -> String {
    let collides = line.trim() == NOTES_MARKER_OPEN || line.trim() == NOTES_MARKER_CLOSE;
    if collides || line.starts_with('\\') {
        format!("\\{line}")
    } else {
        line.to_string()
    }
}

/// Deshace [`escape_sentinel`] solo cuando la marca es inequívoca: o bien el
/// texto tras la marca empieza a su vez por `\` (línea escapada), o bien es el
/// texto exacto de un centinela. Así una línea escrita a mano que empiece por
/// `\` (p. ej. un escape de Markdown) se conserva tal cual.
fn unescape_sentinel(rest: &str) -> String {
    let Some(tail) = rest.strip_prefix('\\') else {
        return rest.to_string();
    };
    let marked = tail.starts_with('\\')
        || tail.trim() == NOTES_MARKER_OPEN
        || tail.trim() == NOTES_MARKER_CLOSE;
    if marked {
        tail.to_string()
    } else {
        rest.to_string()
    }
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
        if escaped {
            escaped = false;
            continue;
        }
        if ch == '\\' {
            escaped = true;
            continue;
        }
        if ch == '"' {
            end = i;
            break;
        }
    }
    if end > 0 {
        Some(rest[..end].to_string())
    } else {
        None
    }
}

impl NotesDb {
    pub fn open(typ_path: &str) -> Result<Self, Box<dyn std::error::Error>> {
        let p = std::path::Path::new(typ_path);
        let db_path = p.with_extension("typshow.db");

        let conn = Connection::open(&db_path)?;
        conn.execute_batch("PRAGMA journal_mode=WAL")?;

        let db = Self {
            conn,
            path: db_path,
        };
        db.init_schema()?;
        Ok(db)
    }

    fn init_schema(&self) -> Result<(), Box<dyn std::error::Error>> {
        self.conn.execute_batch(
            "CREATE TABLE IF NOT EXISTS notes (
                page_number INTEGER PRIMARY KEY,
                heading     TEXT,
                content     TEXT NOT NULL DEFAULT '',
                updated_at  TEXT NOT NULL DEFAULT (datetime('now')),
                source_hash TEXT,
                edited      INTEGER NOT NULL DEFAULT 0,
                heading_edited INTEGER NOT NULL DEFAULT 0
            );",
        )?;
        // Migración idempotente para bases de datos creadas antes de que
        // existieran estas columnas: añade cada una solo si falta, preservando
        // todas las filas.
        self.ensure_column("source_hash", "TEXT")?;
        self.ensure_column("edited", "INTEGER NOT NULL DEFAULT 0")?;
        self.ensure_column("heading_edited", "INTEGER NOT NULL DEFAULT 0")?;
        Ok(())
    }

    /// Añade `name` a `notes` con la declaración `decl` si la columna no existe.
    fn ensure_column(&self, name: &str, decl: &str) -> Result<(), Box<dyn std::error::Error>> {
        let exists: bool = self.conn.query_row(
            "SELECT COUNT(*) FROM pragma_table_info('notes') WHERE name = ?1",
            params![name],
            |r| r.get::<_, i64>(0).map(|n| n > 0),
        )?;
        if !exists {
            // `name` y `decl` son literales internos, no entrada de usuario.
            self.conn
                .execute(&format!("ALTER TABLE notes ADD COLUMN {name} {decl}"), [])?;
        }
        Ok(())
    }

    pub fn path(&self) -> &std::path::Path {
        &self.path
    }

    pub fn has_data(&self) -> Result<bool, Box<dyn std::error::Error>> {
        let count: i64 = self
            .conn
            .query_row("SELECT COUNT(*) FROM notes", [], |r| r.get(0))?;
        Ok(count > 0)
    }

    pub fn load(&self, page: usize) -> Result<NoteRecord, Box<dyn std::error::Error>> {
        let mut stmt = self.conn.prepare(
            "SELECT page_number, heading, content, source_hash, edited
             FROM notes WHERE page_number = ?1",
        )?;

        let result = stmt.query_row(params![page as i64], |r| {
            Ok(NoteRecord {
                page_number: r.get::<_, i64>(0)? as usize,
                heading: r.get(1)?,
                content: r.get(2)?,
                source_hash: r.get(3)?,
                edited: r.get::<_, i64>(4)? != 0,
            })
        });

        match result {
            Ok(record) => Ok(record),
            Err(rusqlite::Error::QueryReturnedNoRows) => Ok(NoteRecord {
                page_number: page,
                heading: None,
                content: String::new(),
                source_hash: None,
                edited: false,
            }),
            Err(e) => Err(Box::new(e)),
        }
    }

    /// Devuelve todas las notas de la base de datos ordenadas por página.
    pub fn all_records(&self) -> Result<Vec<NoteRecord>, Box<dyn std::error::Error>> {
        let mut stmt = self.conn.prepare(
            "SELECT page_number, heading, content, source_hash, edited
             FROM notes ORDER BY page_number",
        )?;
        let rows = stmt.query_map([], |r| {
            Ok(NoteRecord {
                page_number: r.get::<_, i64>(0)? as usize,
                heading: r.get(1)?,
                content: r.get(2)?,
                source_hash: r.get(3)?,
                edited: r.get::<_, i64>(4)? != 0,
            })
        })?;
        let mut records = Vec::new();
        for row in rows {
            records.push(row?);
        }
        Ok(records)
    }

    /// Guarda una nota editada por el usuario; marca `edited = 1`.
    pub fn save(&self, page: usize, content: &str) -> Result<(), Box<dyn std::error::Error>> {
        self.conn.execute(
            "INSERT INTO notes (page_number, content, edited, updated_at)
             VALUES (?1, ?2, 1, datetime('now'))
             ON CONFLICT(page_number) DO UPDATE SET
                content = excluded.content,
                edited = 1,
                updated_at = datetime('now')",
            params![page as i64, content],
        )?;
        Ok(())
    }

    /// Siembra o refresca una nota procedente de un bloque embebido: fija el
    /// contenido, su `source_hash` y `edited = 0` (sin ediciones locales).
    pub fn save_seeded(
        &self,
        page: usize,
        content: &str,
        source_hash: Option<&str>,
    ) -> Result<(), Box<dyn std::error::Error>> {
        self.conn.execute(
            "INSERT INTO notes (page_number, content, source_hash, edited, updated_at)
             VALUES (?1, ?2, ?3, 0, datetime('now'))
             ON CONFLICT(page_number) DO UPDATE SET
                content = excluded.content,
                source_hash = excluded.source_hash,
                edited = 0,
                updated_at = datetime('now')",
            params![page as i64, content, source_hash],
        )?;
        Ok(())
    }

    /// Estado de siembra de una página: `(source_hash, edited)` o `None` si la
    /// página no tiene fila.
    fn row_state(&self, page: usize) -> Result<Option<SeedState>, Box<dyn std::error::Error>> {
        let mut stmt = self.conn.prepare(
            "SELECT heading, heading_edited, source_hash, edited FROM notes WHERE page_number = ?1",
        )?;
        let mut rows = stmt.query(params![page as i64])?;
        match rows.next()? {
            Some(r) => Ok(Some(SeedState {
                heading: r.get(0)?,
                heading_edited: r.get::<_, i64>(1)? != 0,
                source_hash: r.get(2)?,
                edited: r.get::<_, i64>(3)? != 0,
            })),
            None => Ok(None),
        }
    }

    /// Guarda el heading editado por el usuario, protegiéndolo de la
    /// reimportación (`heading_edited = 1`).
    pub fn save_user_heading(
        &self,
        page: usize,
        heading: Option<&str>,
    ) -> Result<(), Box<dyn std::error::Error>> {
        self.conn.execute(
            "INSERT INTO notes (page_number, heading, heading_edited, updated_at)
             VALUES (?1, ?2, 1, datetime('now'))
             ON CONFLICT(page_number) DO UPDATE SET
                heading = excluded.heading,
                heading_edited = 1,
                updated_at = datetime('now')",
            params![page as i64, heading],
        )?;
        Ok(())
    }

    pub fn set_heading(
        &self,
        page: usize,
        heading: Option<&str>,
    ) -> Result<(), Box<dyn std::error::Error>> {
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

    pub fn migrate_from_typ(
        &self,
        typ_content: &str,
        total_pages: usize,
    ) -> Result<(), Box<dyn std::error::Error>> {
        if self.has_data()? {
            // Reimportación selectiva: nunca se vacía la tabla.
            return self.sync_from_blocks(typ_content, total_pages);
        }
        if total_pages == 0 {
            return Ok(());
        }

        let seeds = build_page_seeds(typ_content, total_pages);

        // Page 0: presentation title
        let title = parse_typ_title(typ_content);
        self.set_heading(0, title.as_deref())?;
        self.save_seeded(0, "", None)?;

        for seed in &seeds {
            self.set_heading(seed.page, seed.heading.as_deref())?;
            self.save_seeded(seed.page, &seed.content, seed.source_hash.as_deref())?;
        }

        Ok(())
    }

    /// Refresca selectivamente las páginas sembradas desde bloques embebidos.
    ///
    /// Solo se toca el contenido de las páginas cuyo bloque cambió y que no
    /// tengan ediciones locales; las editadas se conservan y se registra la
    /// divergencia. Nunca se borran filas y, si nada cambió, no se escribe nada.
    fn sync_from_blocks(
        &self,
        typ_content: &str,
        total_pages: usize,
    ) -> Result<(), Box<dyn std::error::Error>> {
        let seeds = build_page_seeds(typ_content, total_pages);

        for seed in &seeds {
            match self.row_state(seed.page)? {
                // Página sin fila: solo se crea si hay un bloque que sembrar.
                None => {
                    let Some(hash) = seed.source_hash.as_deref() else {
                        continue;
                    };
                    self.set_heading(seed.page, seed.heading.as_deref())?;
                    self.save_seeded(seed.page, &seed.content, Some(hash))?;
                }
                Some(state) => {
                    // El heading se refresca desde el documento salvo que el
                    // usuario lo haya editado a mano.
                    if state.heading != seed.heading && !state.heading_edited {
                        self.set_heading(seed.page, seed.heading.as_deref())?;
                    }

                    match (seed.source_hash.as_deref(), state.source_hash.as_deref()) {
                        // El bloque sigue ahí: refrescar solo si cambió el hash.
                        (Some(hash), _) => {
                            if state.source_hash.as_deref() == Some(hash) {
                                continue;
                            }
                            if state.edited {
                                warn!(
                                    "divergencia en la página {}: la nota fue editada localmente y \
                                     el bloque embebido cambió; se conserva la nota",
                                    seed.page + 1
                                );
                            } else {
                                self.save_seeded(seed.page, &seed.content, Some(hash))?;
                            }
                        }
                        // El bloque desapareció del documento: la nota sembrada
                        // queda obsoleta.
                        (None, Some(_)) => {
                            if state.edited {
                                warn!(
                                    "divergencia en la página {}: la nota fue editada localmente y \
                                     su bloque embebido desapareció del documento; se conserva la \
                                     nota",
                                    seed.page + 1
                                );
                            } else {
                                self.save_seeded(seed.page, "", None)?;
                            }
                        }
                        // Nunca hubo bloque: nada que hacer.
                        (None, None) => {}
                    }
                }
            }
        }

        Ok(())
    }
}

/// Hash FNV-1a de 64 bits del Markdown de un bloque, en hexadecimal.
///
/// Se implementa aquí (en lugar de `DefaultHasher`) porque el hash se persiste
/// en `source_hash`: `DefaultHasher` no garantiza el mismo resultado entre
/// versiones de Rust, y eso invalidaría los hashes ya guardados.
pub(crate) fn content_hash(content: &str) -> String {
    const OFFSET_BASIS: u64 = 0xcbf2_9ce4_8422_2325;
    const PRIME: u64 = 0x0000_0100_0000_01b3;

    let mut hash = OFFSET_BASIS;
    for byte in content.as_bytes() {
        hash ^= u64::from(*byte);
        hash = hash.wrapping_mul(PRIME);
    }
    format!("{hash:016x}")
}

/// Estado de siembra de una fila de nota: heading derivado (y si fue editado a
/// mano), hash del bloque del que se sembró y si tiene ediciones locales.
struct SeedState {
    heading: Option<String>,
    heading_edited: bool,
    source_hash: Option<String>,
    edited: bool,
}

/// Plan de sembrado de una página: heading derivado del documento y contenido a
/// escribir, con el hash del bloque embebido cuando procede de uno.
struct PageSeed {
    page: usize,
    heading: Option<String>,
    content: String,
    /// `Some(hash)` cuando `content` viene de un bloque embebido.
    source_hash: Option<String>,
}

/// Calcula, página a página, el heading y el contenido que corresponde sembrar.
///
/// Reproduce la asignación posicional histórica (un heading `=`/`==` por página,
/// reparto de los `===` entre las restantes). Si el documento trae bloques
/// embebidos, el contenido de la página pasa a ser el Markdown de su bloque
/// anclado (o vacío si no tiene) en lugar del contenido de la diapositiva.
fn build_page_seeds(typ_content: &str, total_pages: usize) -> Vec<PageSeed> {
    let mut seeds = Vec::new();
    if total_pages <= 1 {
        return seeds;
    }

    let segments = extract_typ_segments(typ_content);
    let blocks = parse_embedded_blocks(typ_content);
    let has_blocks = !blocks.is_empty();

    if segments.is_empty() {
        for page in 1..total_pages {
            seeds.push(PageSeed {
                page,
                heading: None,
                content: String::new(),
                source_hash: None,
            });
        }
        return seeds;
    }

    let fixed: Vec<&TypSegment> = segments
        .iter()
        .filter(|s| s.heading_level >= 1 && s.heading_level <= 2)
        .collect();
    let flex: Vec<&TypSegment> = segments.iter().filter(|s| s.heading_level >= 3).collect();

    let mut page = 1;

    // Assign one page per = or == heading
    for seg in &fixed {
        if page >= total_pages {
            break;
        }
        let (content, source_hash) = if has_blocks {
            match block_for_heading_line(&blocks, seg.start_line) {
                Some(block) => (block.markdown.clone(), Some(content_hash(&block.markdown))),
                None => (String::new(), None),
            }
        } else {
            (seg.content.clone(), None)
        };
        seeds.push(PageSeed {
            page,
            heading: seg.heading.clone(),
            content,
            source_hash,
        });
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
                seeds.push(PageSeed {
                    page: p,
                    heading: seg.heading.clone(),
                    content: if has_blocks {
                        String::new()
                    } else {
                        seg.content.clone()
                    },
                    source_hash: None,
                });
            }
            page = seg_end;
        }
    } else if page < total_pages {
        let last_seg = segments.last();
        for p in page..total_pages {
            match last_seg {
                Some(seg) => seeds.push(PageSeed {
                    page: p,
                    heading: seg.heading.clone(),
                    content: if has_blocks {
                        String::new()
                    } else {
                        seg.content.clone()
                    },
                    source_hash: None,
                }),
                None => seeds.push(PageSeed {
                    page: p,
                    heading: None,
                    content: String::new(),
                    source_hash: None,
                }),
            }
        }
    }

    seeds
}

/// Primer bloque anclado a la línea de heading dada.
///
/// El emparejamiento es **posicional** (por línea del heading ancla), de modo que
/// dos títulos con el mismo texto no desvían el bloque a la página equivocada.
fn block_for_heading_line(blocks: &[EmbeddedBlock], heading_line: usize) -> Option<&EmbeddedBlock> {
    blocks
        .iter()
        .find(|block| block.heading_line == heading_line)
}

/// Codifica un Markdown como líneas de comentario del bloque embebido.
///
/// Cada línea se emite como `// <línea>`; las vacías como `//`, de modo que el
/// Markdown (incluidas sus líneas en blanco) se recupera íntegro al releerlo. Las
/// líneas que colisionarían con un centinela se escapan (ver [`escape_sentinel`]).
fn encode_block(markdown: &str) -> Vec<String> {
    let mut out = Vec::with_capacity(markdown.lines().count() + 2);
    out.push(NOTES_BLOCK_OPEN.to_string());
    for line in markdown.lines() {
        let escaped = escape_sentinel(line);
        if escaped.is_empty() {
            out.push("//".to_string());
        } else {
            out.push(format!("// {escaped}"));
        }
    }
    out.push(NOTES_BLOCK_CLOSE.to_string());
    out
}

/// Final de línea dominante del fichero (`\r\n` si la mayoría de saltos son CRLF).
///
/// Sirve para no normalizar un `.typ` con CRLF al reescribirlo: el export debe
/// preservar el resto del fichero tal cual estaba.
fn dominant_line_ending(source: &str) -> &'static str {
    let crlf = source.matches("\r\n").count();
    let lf = source.matches('\n').count();
    if crlf > 0 && crlf * 2 >= lf {
        "\r\n"
    } else {
        "\n"
    }
}

/// Reescribe `source` insertando, actualizando o eliminando los bloques de guion
/// embebidos según `records`, preservando el resto del fichero.
///
/// Para cada página con nota no vacía escribe (o actualiza) su bloque justo tras
/// el heading de slide; para las páginas sin nota elimina el bloque existente.
/// Las páginas sin heading asociado (por ejemplo la portada) se ignoran.
pub(crate) fn render_exported_typ(source: &str, records: &[NoteRecord]) -> String {
    use std::collections::{HashMap, HashSet};

    let lines: Vec<&str> = source.lines().collect();
    let headings = parse_typ_headings(source);
    let fixed_lines: Vec<usize> = headings
        .iter()
        .filter(|h| h.level >= 1 && h.level <= 2)
        .map(|h| h.line)
        .collect();

    let page_content: HashMap<usize, &str> = records
        .iter()
        .map(|r| (r.page_number, r.content.as_str()))
        .collect();

    let note_for_page = |page: usize| -> Option<String> {
        let content = page_content.get(&page)?;
        let trimmed = content.trim_end_matches(['\n', '\r']);
        if trimmed.trim().is_empty() {
            None
        } else {
            Some(trimmed.to_string())
        }
    };

    // Página a la que pertenece cada bloque: la de su heading ancla, por
    // posición (nunca reasociando por texto).
    let page_of_block = |block: &EmbeddedBlock| -> Option<usize> {
        fixed_lines
            .iter()
            .position(|line| *line == block.heading_line)
            .map(|i| i + 1)
    };

    // Plan de reescritura: línea de apertura -> (línea de cierre, Markdown a emitir).
    // `None` como Markdown significa eliminar el bloque.
    let mut plan: HashMap<usize, (usize, Option<String>)> = HashMap::new();
    let mut pages_with_block: HashSet<usize> = HashSet::new();
    let mut pages_written: HashSet<usize> = HashSet::new();

    for block in parse_embedded_blocks(source) {
        let Some(page) = page_of_block(&block) else {
            // No debería ocurrir (todo ancla es un heading `=`/`==`): se deja el
            // bloque intacto en lugar de destruirlo.
            continue;
        };
        pages_with_block.insert(page);
        // Solo el primer bloque de la página emite contenido; los duplicados se
        // eliminan.
        let emit = if pages_written.insert(page) {
            note_for_page(page)
        } else {
            None
        };
        plan.insert(block.start_line, (block.end_line, emit));
    }

    let mut out: Vec<String> = Vec::with_capacity(lines.len());
    let mut i = 0;
    while i < lines.len() {
        if let Some((end, emit)) = plan.get(&i) {
            if let Some(markdown) = emit {
                out.extend(encode_block(markdown));
            }
            i = end + 1;
            continue;
        }

        out.push(lines[i].to_string());

        if let Some(pos) = fixed_lines.iter().position(|line| *line == i) {
            let page = pos + 1;
            if !pages_with_block.contains(&page)
                && let Some(markdown) = note_for_page(page)
            {
                out.extend(encode_block(&markdown));
            }
        }

        i += 1;
    }

    let mut result = out.join(dominant_line_ending(source));
    if source.ends_with('\n') {
        result.push_str(dominant_line_ending(source));
    }
    result
}

/// Vuelca las notas de `records` a los bloques embebidos del `.typ` en `path`.
///
/// Escribe el fichero solo si el contenido cambia, preservando byte a byte el
/// resto del documento (ver [`render_exported_typ`]).
pub fn export_to_typ(path: &str, records: &[NoteRecord]) -> Result<(), Box<dyn std::error::Error>> {
    let source = std::fs::read_to_string(path)?;
    let rendered = render_exported_typ(&source, records);
    if rendered != source {
        std::fs::write(path, rendered)?;
    }
    Ok(())
}

/// Cuerpos de las secciones `## …` de un guion Markdown, en orden de aparición.
///
/// El cuerpo de cada sección excluye la línea del propio `## …` y se recorta de
/// líneas en blanco iniciales y finales. El texto previo a la primera sección se
/// ignora.
pub(crate) fn script_section_bodies(markdown: &str) -> Vec<String> {
    let mut sections = Vec::new();
    let mut current: Option<Vec<&str>> = None;

    for line in markdown.lines() {
        if line.trim_start().starts_with("## ") {
            if let Some(body) = current.take() {
                sections.push(trim_body(&body));
            }
            current = Some(Vec::new());
        } else if let Some(body) = current.as_mut() {
            body.push(line);
        }
    }

    if let Some(body) = current.take() {
        sections.push(trim_body(&body));
    }

    sections
}

/// Une las líneas de una sección recortando las líneas en blanco de los extremos.
fn trim_body(lines: &[&str]) -> String {
    let mut start = 0;
    let mut end = lines.len();
    while start < end && lines[start].trim().is_empty() {
        start += 1;
    }
    while end > start && lines[end - 1].trim().is_empty() {
        end -= 1;
    }
    lines[start..end].join("\n")
}

/// Convierte un guion Markdown en bloques embebidos dentro de `typ_source`.
///
/// Empareja la n-ésima sección `## …` con el n-ésimo heading de slide (`=`/`==`),
/// en orden, reutilizando el export idempotente: reejecutar actualiza los bloques
/// existentes en lugar de duplicarlos. Devuelve el `.typ` resultante y cuántas
/// secciones se escribieron.
pub(crate) fn embed_script_into_typ(script: &str, typ_source: &str) -> (String, usize) {
    let bodies = script_section_bodies(script);
    let heading_count = parse_typ_headings(typ_source)
        .iter()
        .filter(|h| h.level >= 1 && h.level <= 2)
        .count();

    let written = bodies.len().min(heading_count);
    if bodies.len() > heading_count {
        warn!(
            "el guion tiene {} secciones pero el documento solo {heading_count} headings de \
             slide; se ignoran {}",
            bodies.len(),
            bodies.len() - heading_count
        );
    }

    let records: Vec<NoteRecord> = bodies
        .iter()
        .take(written)
        .enumerate()
        .map(|(i, body)| NoteRecord {
            page_number: i + 1,
            heading: None,
            content: body.clone(),
            source_hash: None,
            edited: false,
        })
        .collect();

    let written = records
        .iter()
        .filter(|record| !record.content.trim().is_empty())
        .count();

    (render_exported_typ(typ_source, &records), written)
}

/// Subcomando de conversión: pasa las secciones `## …` de `script_path` a bloques
/// embebidos dentro del `.typ` de `typ_path`. Devuelve cuántos bloques se escribieron.
pub fn guion_embed_files(
    script_path: &str,
    typ_path: &str,
) -> Result<usize, Box<dyn std::error::Error>> {
    let script = std::fs::read_to_string(script_path)?;
    let typ = std::fs::read_to_string(typ_path)?;
    let (rendered, written) = embed_script_into_typ(&script, &typ);
    if rendered != typ {
        std::fs::write(typ_path, rendered)?;
    }
    Ok(written)
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
        let end = boundaries
            .get(i + 1)
            .map(|(l, _)| *l)
            .unwrap_or(total_lines);

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
        let level = if trimmed.starts_with("==== ") {
            4
        } else if trimmed.starts_with("=== ") {
            3
        } else if trimmed.starts_with("== ") {
            2
        } else if trimmed.starts_with("= ") {
            1
        } else {
            continue;
        };

        let text = trimmed[level..].trim().to_string();
        headings.push(HeadingInfo {
            level: level as u32,
            text,
            line: i,
        });
    }
    headings
}

pub fn extract_typ_segments(source: &str) -> Vec<TypSegment> {
    let headings = parse_typ_headings(source);
    let lines: Vec<&str> = source.lines().collect();
    let total_lines = lines.len();
    let block_ranges = embedded_block_ranges(&lines);
    let in_embedded_block = |index: usize| {
        block_ranges
            .iter()
            .any(|(start, end)| index >= *start && index <= *end)
    };

    let mut boundaries: Vec<(usize, Option<String>, u32)> = Vec::new();
    for h in &headings {
        if !boundaries.iter().any(|(l, _, _)| *l == h.line) {
            boundaries.push((h.line, Some(h.text.clone()), h.level));
        }
    }
    for (i, line) in lines.iter().enumerate() {
        if line.trim().starts_with("#pagebreak") && !boundaries.iter().any(|(l, _, _)| *l == i) {
            boundaries.push((i, None, 0));
        }
    }
    boundaries.sort_by_key(|(line, _, _)| *line);

    let mut segments = Vec::new();
    let mut last_label: Option<String> = None;

    for i in 0..boundaries.len() {
        let (start, label_opt, level) = &boundaries[i];
        let end = boundaries
            .get(i + 1)
            .map(|(l, _, _)| *l)
            .unwrap_or(total_lines);

        let resolved = label_opt.clone().or_else(|| last_label.clone());
        if label_opt.is_some() {
            last_label = label_opt.clone();
        }

        let content: String = lines[*start..end]
            .iter()
            .enumerate()
            .filter(|(offset, _)| !in_embedded_block(*start + *offset))
            .map(|(_, l)| *l)
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
            HeadingInfo {
                level: 1,
                text: "A".into(),
                line: 0,
            },
            HeadingInfo {
                level: 1,
                text: "B".into(),
                line: 10,
            },
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

    #[test]
    fn parse_embedded_block_basico() {
        let src = "== Hubo un día\n\n// >>> typshow:notes\n// [0:00 – 1:00]\n//\n// El guion.\n// >>> typshow:notes:end\n";
        let blocks = parse_embedded_blocks(src);
        assert_eq!(blocks.len(), 1);
        assert_eq!(blocks[0].heading, "Hubo un día");
        assert_eq!(blocks[0].markdown, "[0:00 – 1:00]\n\nEl guion.");
        assert_eq!(blocks[0].start_line, 2);
        assert_eq!(blocks[0].end_line, 6);
    }

    #[test]
    fn parse_embedded_block_sin_cerrar() {
        let src = "== A\n// >>> typshow:notes\n// texto sin cerrar\n";
        assert!(parse_embedded_blocks(src).is_empty());
    }

    #[test]
    fn parse_embedded_blocks_anclaje() {
        let src = "== Uno\n// >>> typshow:notes\n// nota uno\n// >>> typshow:notes:end\n\
                   == Dos\n// >>> typshow:notes\n// nota dos\n// >>> typshow:notes:end\n";
        let blocks = parse_embedded_blocks(src);
        assert_eq!(blocks.len(), 2);
        assert_eq!(blocks[0].heading, "Uno");
        assert_eq!(blocks[0].markdown, "nota uno");
        assert_eq!(blocks[1].heading, "Dos");
        assert_eq!(blocks[1].markdown, "nota dos");
    }

    #[test]
    fn parse_embedded_blocks_sin_heading() {
        let src = "// >>> typshow:notes\n// huérfano\n// >>> typshow:notes:end\n= Después\n";
        assert!(parse_embedded_blocks(src).is_empty());
    }

    #[test]
    fn parse_embedded_blocks_ignora_heading_profundo() {
        // Un `===` no ancla: solo `=` y `==` son headings de slide.
        let src = "=== Detalle\n// >>> typshow:notes\n// texto\n// >>> typshow:notes:end\n";
        assert!(parse_embedded_blocks(src).is_empty());
    }

    #[test]
    fn parse_embedded_block_ultimo_gana() {
        // El ancla es el heading inmediatamente anterior, no el primero.
        let src = "== Uno\n== Dos\n// >>> typshow:notes\n// texto\n// >>> typshow:notes:end\n";
        let blocks = parse_embedded_blocks(src);
        assert_eq!(blocks.len(), 1);
        assert_eq!(blocks[0].heading, "Dos");
    }

    #[test]
    fn bloque_anidado_descarta_el_exterior() {
        let src = "== A\n\
                   // >>> typshow:notes\n\
                   // exterior\n\
                   // >>> typshow:notes\n\
                   // interior\n\
                   // >>> typshow:notes:end\n";
        let blocks = parse_embedded_blocks(src);
        assert_eq!(blocks.len(), 1);
        assert_eq!(blocks[0].markdown, "interior");
        assert_eq!(blocks[0].start_line, 3, "vale el centinela interior");
        assert_eq!(blocks[0].heading_line, 0);

        // Y el bloque exterior descartado tampoco se cuela en el segmento.
        let segments = extract_typ_segments(src);
        assert!(
            segments
                .iter()
                .all(|s| !s.content.contains("exterior") && !s.content.contains("interior"))
        );
    }

    #[test]
    fn bloques_con_headings_repetidos_va_a_su_pagina() {
        // Dos `== Repaso`: el bloque del segundo va a la segunda página, no a la
        // primera (el emparejamiento debe ser posicional, no por texto).
        let src = "== Repaso\n\
                   == Repaso\n\
                   // >>> typshow:notes\n\
                   // segundo\n\
                   // >>> typshow:notes:end\n";
        let blocks = parse_embedded_blocks(src);
        assert_eq!(blocks.len(), 1);
        assert_eq!(blocks[0].heading_line, 1);

        let db = test_db();
        db.migrate_from_typ(src, 3).unwrap();
        assert_eq!(db.load(1).unwrap().heading.as_deref(), Some("Repaso"));
        assert_eq!(db.load(1).unwrap().content, "");
        assert_eq!(db.load(2).unwrap().content, "segundo");
    }

    #[test]
    fn bloques_con_headings_repetidos_en_orden() {
        let src = "== Repaso\n\
                   // >>> typshow:notes\n\
                   // primero\n\
                   // >>> typshow:notes:end\n\
                   == Repaso\n\
                   // >>> typshow:notes\n\
                   // segundo\n\
                   // >>> typshow:notes:end\n\
                   == Repaso\n";
        let db = test_db();
        db.migrate_from_typ(src, 4).unwrap();
        assert_eq!(db.load(1).unwrap().content, "primero");
        assert_eq!(db.load(2).unwrap().content, "segundo");
        assert_eq!(db.load(3).unwrap().content, "");
    }

    const TYP_BLOCKS_V1: &str = "== Uno\nslide uno\n// >>> typshow:notes\n// guion uno\n// >>> typshow:notes:end\n\
                                 == Dos\nslide dos\n// >>> typshow:notes\n//\n// guion dos\n// >>> typshow:notes:end";
    const TYP_BLOCKS_V2: &str = "== Uno\nslide uno\n// >>> typshow:notes\n// guion uno NUEVO\n// >>> typshow:notes:end\n\
                                 == Dos\nslide dos\n// >>> typshow:notes\n//\n// guion dos\n// >>> typshow:notes:end";

    #[test]
    fn migrate_from_blocks() {
        let db = test_db();
        db.migrate_from_typ(TYP_BLOCKS_V1, 3).unwrap();
        assert!(db.has_data().unwrap());
        assert_eq!(db.load(0).unwrap().content, "");
        assert_eq!(db.load(1).unwrap().heading.as_deref(), Some("Uno"));
        assert_eq!(db.load(1).unwrap().content, "guion uno");
        assert_eq!(db.load(2).unwrap().heading.as_deref(), Some("Dos"));
        assert_eq!(db.load(2).unwrap().content, "\nguion dos");
        // Las notas sembradas de bloques no están editadas y conocen su hash.
        assert_eq!(
            db.load(1).unwrap().source_hash,
            Some(content_hash("guion uno"))
        );
        assert!(!db.load(1).unwrap().edited);
        assert!(!db.load(2).unwrap().edited);
    }

    #[test]
    fn migrate_sin_bloques_como_antes() {
        let db = test_db();
        db.migrate_from_typ("== Sec1\ndetailed\ncontent here", 2)
            .unwrap();
        assert_eq!(db.load(1).unwrap().content, "detailed\ncontent here");
        assert_eq!(db.load(1).unwrap().source_hash, None);
        assert!(!db.load(1).unwrap().edited);
    }

    #[test]
    fn sin_fuga_de_comentarios() {
        let src = "== A\nslide text\n// >>> typshow:notes\n// guion secreto\n// >>> typshow:notes:end\nmore slide\n== B\ntexto B";
        let segments = extract_typ_segments(src);
        assert!(
            segments
                .iter()
                .all(|s| !s.content.contains("guion secreto")),
            "el guion no debe filtrarse a los segmentos: {:?}",
            segments.iter().map(|s| &s.content).collect::<Vec<_>>()
        );
        assert_eq!(segments[0].content, "slide text\nmore slide");
        assert_eq!(segments[1].content, "texto B");

        // Tampoco debe aparecer como nota sembrada.
        let db = test_db();
        db.migrate_from_typ(src, 3).unwrap();
        assert!(
            db.all_records()
                .unwrap()
                .iter()
                .all(|r| !r.content.contains("slide text"))
        );
        assert_eq!(db.load(1).unwrap().content, "guion secreto");
    }

    #[test]
    fn migracion_esquema_hash() {
        // Base de datos "antigua" con el esquema previo y una fila.
        let conn = Connection::open_in_memory().unwrap();
        conn.execute_batch(
            "CREATE TABLE notes (
                page_number INTEGER PRIMARY KEY,
                heading     TEXT,
                content     TEXT NOT NULL DEFAULT '',
                updated_at  TEXT NOT NULL DEFAULT (datetime('now'))
            );
            INSERT INTO notes (page_number, heading, content) VALUES (0, 'Vieja', 'contenido previo');",
        )
        .unwrap();
        let db = NotesDb {
            conn,
            path: PathBuf::from(":memory:"),
        };

        db.init_schema().unwrap();

        let cols: i64 = db
            .conn
            .query_row(
                "SELECT COUNT(*) FROM pragma_table_info('notes')
                 WHERE name IN ('source_hash', 'edited', 'heading_edited')",
                [],
                |r| r.get(0),
            )
            .unwrap();
        assert_eq!(cols, 3, "todas las columnas deben existir tras migrar");

        // La fila existente se preserva íntegra.
        let rec = db.load(0).unwrap();
        assert_eq!(rec.heading.as_deref(), Some("Vieja"));
        assert_eq!(rec.content, "contenido previo");
        assert_eq!(rec.source_hash, None);
        assert!(!rec.edited);

        // Idempotente: repetir no falla ni duplica nada.
        db.init_schema().unwrap();
        assert_eq!(db.all_records().unwrap().len(), 1);
    }

    #[test]
    fn reimport_refresca_no_editada() {
        let db = test_db();
        db.migrate_from_typ(TYP_BLOCKS_V1, 3).unwrap();
        assert_eq!(db.load(1).unwrap().content, "guion uno");

        db.migrate_from_typ(TYP_BLOCKS_V2, 3).unwrap();

        assert_eq!(db.load(1).unwrap().content, "guion uno NUEVO");
        assert_eq!(
            db.load(1).unwrap().source_hash,
            Some(content_hash("guion uno NUEVO"))
        );
        assert!(!db.load(1).unwrap().edited);
    }

    #[test]
    fn reimport_respeta_editada() {
        let db = test_db();
        db.migrate_from_typ(TYP_BLOCKS_V1, 3).unwrap();
        db.save(1, "mi edición local").unwrap();
        assert!(db.load(1).unwrap().edited);

        db.migrate_from_typ(TYP_BLOCKS_V2, 3).unwrap();

        assert_eq!(db.load(1).unwrap().content, "mi edición local");
        assert!(db.load(1).unwrap().edited);
    }

    #[test]
    fn reimport_sin_cambios() {
        let db = test_db();
        db.migrate_from_typ(TYP_BLOCKS_V1, 3).unwrap();
        let before = db.all_records().unwrap();
        let stamps: Vec<String> = db
            .all_records()
            .unwrap()
            .iter()
            .map(|r| {
                db.conn
                    .query_row(
                        "SELECT updated_at FROM notes WHERE page_number = ?1",
                        params![r.page_number as i64],
                        |row| row.get::<_, String>(0),
                    )
                    .unwrap()
            })
            .collect();

        db.migrate_from_typ(TYP_BLOCKS_V1, 3).unwrap();

        let after = db.all_records().unwrap();
        assert_eq!(before.len(), after.len());
        for (b, a) in before.iter().zip(after.iter()) {
            assert_eq!(b.page_number, a.page_number);
            assert_eq!(b.heading, a.heading);
            assert_eq!(b.content, a.content);
            assert_eq!(b.source_hash, a.source_hash);
            assert_eq!(b.edited, a.edited);
        }
        // Ninguna fila se toca: ni siquiera `updated_at` cambia.
        for (stamp, record) in stamps.iter().zip(after.iter()) {
            let now: String = db
                .conn
                .query_row(
                    "SELECT updated_at FROM notes WHERE page_number = ?1",
                    params![record.page_number as i64],
                    |row| row.get(0),
                )
                .unwrap();
            assert_eq!(
                stamp, &now,
                "la fila {} no debe reescribirse",
                record.page_number
            );
        }
    }

    #[test]
    fn reimport_no_destructiva() {
        let db = test_db();
        db.migrate_from_typ(TYP_BLOCKS_V1, 3).unwrap();
        db.save(0, "nota del título").unwrap();
        db.save(1, "editada a mano").unwrap();
        db.save(2, "página extra").unwrap();

        db.migrate_from_typ(TYP_BLOCKS_V2, 3).unwrap();

        assert!(db.has_data().unwrap());
        assert_eq!(db.load(0).unwrap().content, "nota del título");
        assert_eq!(db.load(1).unwrap().content, "editada a mano");
        assert_eq!(db.load(2).unwrap().content, "página extra");
        assert_eq!(db.all_records().unwrap().len(), 3);
    }

    fn note(page: usize, content: &str) -> NoteRecord {
        NoteRecord {
            page_number: page,
            heading: None,
            content: content.to_string(),
            source_hash: None,
            edited: false,
        }
    }

    #[test]
    fn export_inserta_bloque() {
        let src = "== Uno\nslide uno\n\n== Dos\nslide dos\n";
        let out = render_exported_typ(src, &[note(1, "guion uno")]);
        assert_eq!(
            out,
            "== Uno\n\
             // >>> typshow:notes\n\
             // guion uno\n\
             // >>> typshow:notes:end\n\
             slide uno\n\n== Dos\nslide dos\n"
        );
        // El bloque generado es releíble y conserva el Markdown.
        let blocks = parse_embedded_blocks(&out);
        assert_eq!(blocks.len(), 1);
        assert_eq!(blocks[0].heading, "Uno");
        assert_eq!(blocks[0].markdown, "guion uno");
    }

    #[test]
    fn export_actualiza_bloque() {
        let src = "= Intro\n\
                   // >>> typshow:notes\n\
                   // viejo\n\
                   // >>> typshow:notes:end\n\
                   code after\n";
        let out = render_exported_typ(src, &[note(1, "nuevo\n\ndos líneas")]);
        assert_eq!(
            out,
            "= Intro\n\
             // >>> typshow:notes\n\
             // nuevo\n\
             //\n\
             // dos líneas\n\
             // >>> typshow:notes:end\n\
             code after\n"
        );
    }

    #[test]
    fn export_elimina_bloque_vacio() {
        let src = "= Intro\n\
                   // >>> typshow:notes\n\
                   // viejo\n\
                   // >>> typshow:notes:end\n\
                   code after\n";
        let out = render_exported_typ(src, &[note(1, "")]);
        assert_eq!(out, "= Intro\ncode after\n");
    }

    #[test]
    fn export_preserva_el_resto_del_fichero() {
        let src = "#show: slides.with(title: \"X\")\n\n\
                   == Uno\n\
                   // >>> typshow:notes\n\
                   // viejo\n\
                   // >>> typshow:notes:end\n\n\
                   == Dos\n\
                   // >>> typshow:notes\n\
                   // dos\n\
                   // >>> typshow:notes:end\n";
        let out = render_exported_typ(src, &[note(1, "uno nuevo"), note(2, "dos")]);
        assert!(out.starts_with("#show: slides.with(title: \"X\")\n\n== Uno\n"));
        assert!(out.ends_with("== Dos\n// >>> typshow:notes\n// dos\n// >>> typshow:notes:end\n"));
        assert_eq!(out.matches(">>> typshow:notes").count(), 4);
        // Idempotente: exportar de nuevo no cambia nada.
        assert_eq!(
            render_exported_typ(&out, &[note(1, "uno nuevo"), note(2, "dos")]),
            out
        );
    }

    #[test]
    fn export_to_typ_escribe_el_fichero() {
        let dir = std::env::temp_dir();
        let unique = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .map(|d| d.as_nanos())
            .unwrap_or(0);
        let path = dir.join(format!(
            "typshow-export-{}-{unique}.typ",
            std::process::id()
        ));
        let path_str = path.to_string_lossy().to_string();
        std::fs::write(&path, "== Uno\nslide\n").unwrap();

        export_to_typ(&path_str, &[note(1, "guion")]).unwrap();

        let content = std::fs::read_to_string(&path).unwrap();
        assert!(content.contains("// guion"));
        std::fs::remove_file(&path).ok();
    }

    #[test]
    fn conversor_por_orden() {
        let script = "## Uno\n\nguion uno\n\n## Dos\n\nguion dos\n\n## Tres\n\nguion tres\n";
        let typ = "== Uno\nslide uno\n\n== Dos\nslide dos\n\n== Tres\nslide tres\n";

        let (out, written) = embed_script_into_typ(script, typ);

        assert_eq!(written, 3);
        let blocks = parse_embedded_blocks(&out);
        assert_eq!(blocks.len(), 3);
        assert_eq!(blocks[0].heading, "Uno");
        assert_eq!(blocks[0].markdown, "guion uno");
        assert_eq!(blocks[1].heading, "Dos");
        assert_eq!(blocks[1].markdown, "guion dos");
        assert_eq!(blocks[2].heading, "Tres");
        assert_eq!(blocks[2].markdown, "guion tres");

        let pos = |needle: &str| out.find(needle).unwrap();
        assert!(pos("== Uno") < pos("// guion uno"));
        assert!(pos("// guion uno") < pos("== Dos"));
        assert!(pos("== Dos") < pos("// guion dos"));
        assert!(pos("// guion dos") < pos("== Tres"));
        assert!(pos("== Tres") < pos("// guion tres"));
    }

    #[test]
    fn conversor_idempotente() {
        let script = "## Uno\n\nguion uno\n\n## Dos\n\nguion dos\n";
        let typ = "== Uno\nslide uno\n\n== Dos\nslide dos\n";

        let (out1, _) = embed_script_into_typ(script, typ);
        let (out2, _) = embed_script_into_typ(script, &out1);
        let (out3, _) = embed_script_into_typ(script, &out2);

        assert_eq!(out1, out2);
        assert_eq!(out2, out3);
        assert_eq!(out3.matches(">>> typshow:notes").count(), 4);
        assert_eq!(parse_embedded_blocks(&out3).len(), 2);
    }

    #[test]
    fn conversor_ignora_secciones_sobrantes() {
        let script = "## Uno\n\nuno\n\n## Dos\n\ndos\n";
        let typ = "== Uno\nslide\n";
        let (out, written) = embed_script_into_typ(script, typ);
        assert_eq!(written, 1);
        assert_eq!(parse_embedded_blocks(&out).len(), 1);
        assert!(!out.contains("dos"), "la sección sobrante no se escribe");
    }

    #[test]
    fn conversor_ficheros() {
        let dir = std::env::temp_dir();
        let unique = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .map(|d| d.as_nanos())
            .unwrap_or(0);
        let script_path = dir.join(format!("typshow-guion-{}-{unique}.md", std::process::id()));
        let typ_path = dir.join(format!("typshow-guion-{}-{unique}.typ", std::process::id()));
        std::fs::write(&script_path, "## Uno\n\ncuerpo uno\n").unwrap();
        std::fs::write(&typ_path, "== Uno\nslide\n").unwrap();

        let written =
            guion_embed_files(&script_path.to_string_lossy(), &typ_path.to_string_lossy()).unwrap();

        assert_eq!(written, 1);
        let typ = std::fs::read_to_string(&typ_path).unwrap();
        assert!(typ.contains("// cuerpo uno"));
        // Idempotente también a nivel de fichero.
        guion_embed_files(&script_path.to_string_lossy(), &typ_path.to_string_lossy()).unwrap();
        let typ2 = std::fs::read_to_string(&typ_path).unwrap();
        assert_eq!(typ, typ2);
        std::fs::remove_file(&script_path).ok();
        std::fs::remove_file(&typ_path).ok();
    }

    const TYP_SIN_BLOQUE_UNO: &str = "== Uno\nslide uno\n\
                                      == Dos\nslide dos\n\
                                      // >>> typshow:notes\n\
                                      //\n\
                                      // guion dos\n\
                                      // >>> typshow:notes:end";

    #[test]
    fn reimport_limpia_bloque_eliminado() {
        let db = test_db();
        db.migrate_from_typ(TYP_BLOCKS_V1, 3).unwrap();
        assert_eq!(db.load(1).unwrap().content, "guion uno");

        // El bloque de la página 1 desaparece del documento.
        db.migrate_from_typ(TYP_SIN_BLOQUE_UNO, 3).unwrap();

        let uno = db.load(1).unwrap();
        assert_eq!(uno.content, "", "la nota obsoleta se vacía");
        assert_eq!(uno.source_hash, None, "el hash rancio se limpia");
        assert!(!uno.edited);
        // La página 2 conserva su bloque intacto.
        assert_eq!(db.load(2).unwrap().content, "\nguion dos");
    }

    #[test]
    fn reimport_conserva_bloque_eliminado_si_editada() {
        let db = test_db();
        db.migrate_from_typ(TYP_BLOCKS_V1, 3).unwrap();
        db.save(1, "editada a mano").unwrap();

        db.migrate_from_typ(TYP_SIN_BLOQUE_UNO, 3).unwrap();

        let uno = db.load(1).unwrap();
        assert_eq!(uno.content, "editada a mano", "la edición local se respeta");
        assert!(uno.edited);
    }

    #[test]
    fn reimport_respeta_heading_editado() {
        let db = test_db();
        db.migrate_from_typ(TYP_BLOCKS_V1, 3).unwrap();
        db.save_user_heading(1, Some("Mi título")).unwrap();

        db.migrate_from_typ(TYP_BLOCKS_V2, 3).unwrap();

        assert_eq!(db.load(1).unwrap().heading.as_deref(), Some("Mi título"));
    }

    #[test]
    fn reimport_actualiza_heading_no_editado() {
        let db = test_db();
        db.migrate_from_typ("== Uno\n", 2).unwrap();
        assert_eq!(db.load(1).unwrap().heading.as_deref(), Some("Uno"));

        db.migrate_from_typ("== Renombrado\n", 2).unwrap();

        assert_eq!(db.load(1).unwrap().heading.as_deref(), Some("Renombrado"));
    }

    #[test]
    fn conversor_no_cuenta_secciones_vacias() {
        let script = "## Uno\n\nguion uno\n\n## Dos\n\n";
        let typ = "== Uno\nslide\n\n== Dos\nslide\n";

        let (out, written) = embed_script_into_typ(script, typ);

        assert_eq!(written, 1, "la sección vacía no cuenta como escrita");
        assert_eq!(parse_embedded_blocks(&out).len(), 1);
        assert!(out.contains("// guion uno"));
    }

    #[test]
    fn export_preserva_crlf() {
        let src = "== Uno\r\nslide uno\r\n\r\n== Dos\r\nslide dos\r\n";
        let out = render_exported_typ(src, &[note(1, "guion uno")]);
        assert_eq!(
            out,
            "== Uno\r\n\
             // >>> typshow:notes\r\n\
             // guion uno\r\n\
             // >>> typshow:notes:end\r\n\
             slide uno\r\n\r\n== Dos\r\nslide dos\r\n"
        );
        assert!(
            !out.replace("\r\n", "").contains('\n'),
            "no debe colarse ningún LF suelto"
        );
    }

    #[test]
    fn round_trip_centinelas_escapados() {
        let markdown = "antes\n>>> typshow:notes\n// >>> typshow:notes:end\n\\escape\ndespués";
        let records = [note(1, markdown)];

        let out = render_exported_typ("== Uno\nslide\n", &records);

        let blocks = parse_embedded_blocks(&out);
        assert_eq!(blocks.len(), 1);
        assert_eq!(blocks[0].markdown, markdown, "round-trip exacto: {out}");
        // Idempotente y estable.
        assert_eq!(render_exported_typ(&out, &records), out);
    }

    #[test]
    fn content_hash_es_fnv1a_estable() {
        // Vectores conocidos de FNV-1a 64 bits: pincha el algoritmo para que un
        // cambio accidental no invalide los hashes ya persistidos.
        assert_eq!(content_hash(""), "cbf29ce484222325");
        assert_eq!(content_hash("a"), "af63dc4c8601ec8c");
        assert_eq!(content_hash("guion uno").len(), 16);
    }
}
