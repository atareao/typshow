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
/// ignora. Las líneas `## …` que caen dentro de un bloque de código cercado o que
/// llevan indentación de código (4 o más espacios) NO se consideran secciones.
pub(crate) fn script_section_bodies(markdown: &str) -> Vec<String> {
    let mut sections = Vec::new();
    let mut current: Option<Vec<&str>> = None;

    let fenced = fenced_line_flags(markdown);

    for (line, inside_fence) in markdown.lines().zip(fenced) {
        if !inside_fence && is_section_heading(line) {
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

/// ¿Es `line` una cabecera de sección (`## `, o exactamente `##`) del guion?
///
/// Solo cuentan las cabeceras con indentación de hasta 3 espacios: 4 o más
/// espacios son un bloque de código indentado en Markdown, no una cabecera.
fn is_section_heading(line: &str) -> bool {
    if indent_width(line) > 3 {
        return false;
    }
    let trimmed = line.trim_start();
    trimmed == "##" || trimmed.starts_with("## ")
}

/// Número de espacios de indentación inicial de `line`.
fn indent_width(line: &str) -> usize {
    line.len() - line.trim_start().len()
}

/// Marca de un fence de código: `(carácter, longitud)` si `line` lo abre.
///
/// Solo cuentan los fences de 3 o más backticks o tildes con indentación de
/// hasta 3 espacios. La información tras la marca (p. ej. el lenguaje) es
/// admisible al abrir el fence.
fn fence_marker(line: &str) -> Option<(char, usize)> {
    if indent_width(line) > 3 {
        return None;
    }
    let trimmed = line.trim_start();
    let marker = trimmed.chars().next()?;
    if marker != '`' && marker != '~' {
        return None;
    }
    let len = trimmed.chars().take_while(|c| *c == marker).count();
    (len >= 3).then_some((marker, len))
}

/// Marca si cada línea de `markdown` cae dentro de un bloque de código cercado.
///
/// Un fence abierto se cierra con otro fence del mismo carácter y longitud igual
/// o mayor que no lleve nada tras la marca (la línea de cierre es «desnuda»).
fn fenced_line_flags(markdown: &str) -> Vec<bool> {
    let mut flags = Vec::new();
    let mut open: Option<(char, usize)> = None;

    for line in markdown.lines() {
        match open {
            None => {
                flags.push(false);
                if let Some(marker) = fence_marker(line) {
                    open = Some(marker);
                }
            }
            Some((marker_char, marker_len)) => {
                let closes = fence_marker(line).is_some_and(|(c, len)| {
                    c == marker_char && len >= marker_len && line.trim().chars().all(|ch| ch == c)
                });
                flags.push(!closes);
                if closes {
                    open = None;
                }
            }
        }
    }

    flags
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

/// ¿Es `line` un separador de tramo (una línea de solo `─`, al menos 10)?
fn is_timeline_separator(line: &str) -> bool {
    let trimmed = line.trim();
    trimmed.chars().count() >= 10 && trimmed.chars().all(|c| c == '─')
}

/// Rangos de bytes `(inicio, fin)` de cada marca `H:MM` de la línea.
fn time_mark_ranges(line: &str) -> Vec<(usize, usize)> {
    let bytes = line.as_bytes();
    let mut marks = Vec::new();
    let mut i = 0;
    while i < bytes.len() {
        if !bytes[i].is_ascii_digit() {
            i += 1;
            continue;
        }
        let start = i;
        let mut j = i;
        while j < bytes.len() && bytes[j].is_ascii_digit() && j - start < 2 {
            j += 1;
        }
        let has_clock = bytes.get(j) == Some(&b':')
            && bytes.get(j + 1).is_some_and(u8::is_ascii_digit)
            && bytes.get(j + 2).is_some_and(u8::is_ascii_digit);
        if has_clock {
            marks.push((start, j + 3));
            i = j + 3;
        } else {
            i = start + 1;
        }
    }
    marks
}

/// ¿Es `gap` el separador entre dos marcas horarias contiguas (` – `, ` - `, `~`)?
fn is_time_range_gap(gap: &str) -> bool {
    let gap = gap.trim();
    !gap.is_empty() && gap.len() <= 3 && gap.chars().all(|c| matches!(c, '-' | '–' | '—' | '~'))
}

/// ¿**Abre** `line` un tramo horario `H:MM – H:MM` (formato de cabecera de bloque)?
///
/// Debe empezar por el tramo (solo espacios delante): una línea de cuerpo que
/// mencione horas en medio no es una cabecera.
fn starts_with_time_range(line: &str) -> bool {
    let marks = time_mark_ranges(line);
    let Some(&(start, end)) = marks.first() else {
        return false;
    };
    line[..start].trim().is_empty()
        && marks
            .get(1)
            .is_some_and(|&(second_start, _)| is_time_range_gap(&line[end..second_start]))
}

/// ¿Contiene `line` un tramo horario `H:MM – H:MM` en cualquier posición?
///
/// Sirve para reconocer regiones de meta o cola (p. ej. «duración total
/// 45:00 – 50:00»), que no son cabecera ni cuerpo de bloque.
///
/// Limitación conocida: al aplicarse línea a línea sobre una región, una línea
/// intermedia del cuerpo con un rango horario textual (p. ej. «Repaso: 0:00 – 2:00»)
/// también la clasifica como meta/cola y puede truncar el bloque en curso (ver
/// [`script_timeline_blocks`]).
fn contains_time_range(line: &str) -> bool {
    let marks = time_mark_ranges(line);
    marks
        .windows(2)
        .any(|pair| is_time_range_gap(&line[pair[0].1..pair[1].0]))
}

/// Regiones del guion delimitadas por separadores de línea `─` (≥10).
///
/// La primera región es el preámbulo (lo anterior al primer separador) y las
/// propias líneas de separación no forman parte de ninguna región.
fn timeline_regions(markdown: &str) -> Vec<Vec<&str>> {
    let mut regions: Vec<Vec<&str>> = Vec::new();
    let mut current: Vec<&str> = Vec::new();
    for line in markdown.lines() {
        if is_timeline_separator(line) {
            regions.push(std::mem::take(&mut current));
        } else {
            current.push(line);
        }
    }
    regions.push(current);
    regions
}

/// Trocea un guion de grano grueso en sus bloques, en orden de aparición.
///
/// Un bloque arranca en una **región-cabecera** (su primera línea no vacía empieza
/// por `H:MM – H:MM`) y su contenido es esa región **más las regiones siguientes**
/// que no abren cabecera, de modo que funcionan tanto el cuerpo en la misma región
/// que la cabecera como la estructura real de las escaletas, donde la cabecera va
/// sola entre dos separadores y el cuerpo llega en la región siguiente:
///
/// ```text
/// ──────  ──────  ──────────────
/// 0:00 – 2:00  🎬 INTRO        ── región-cabecera   } bloque
/// ──────  ──────               ── región-cuerpo     }
/// 📌 Puntos a tratar: …
/// ```
///
/// El preámbulo anterior al primer bloque se ignora. Una región posterior con un
/// tramo horario en medio (meta o cola, p. ej. una nota de duración) cierra el
/// bloque y se ignora. Un guion sin separadores devuelve vacío.
///
/// Limitaciones conocidas:
///
/// - La cola de producción **sin** marca horaria (p. ej. un `CHECKLIST`) se anexa
///   al cuerpo del **último** bloque: el guion no trae marca explícita de fin.
/// - Una **línea intermedia** del cuerpo con un rango horario textual (p. ej.
///   «Repaso: 0:00 – 2:00») puede truncar el bloque y descartar el resto, porque
///   la detección de fin de bloque usa [`contains_time_range`].
pub(crate) fn script_timeline_blocks(markdown: &str) -> Vec<String> {
    let mut blocks = Vec::new();
    let mut current: Option<Vec<&str>> = None;

    let flush = |current: &mut Option<Vec<&str>>, blocks: &mut Vec<String>| {
        if let Some(body) = current.take() {
            blocks.push(trim_body(&body));
        }
    };

    for region in timeline_regions(markdown) {
        let opens_block = region
            .iter()
            .find(|line| !line.trim().is_empty())
            .is_some_and(|header| starts_with_time_range(header));

        if opens_block {
            flush(&mut current, &mut blocks);
            current = Some(region);
        } else if region.iter().any(|line| contains_time_range(line)) {
            // Meta o cola: cierra el bloque y no abre uno nuevo.
            flush(&mut current, &mut blocks);
        } else if let Some(body) = current.as_mut() {
            body.extend(region);
        }
    }
    flush(&mut current, &mut blocks);

    blocks
}

/// Número de headings ancla (`=`/`==`) y posiciones (0-based) de las secciones `=`.
///
/// Reproduce el reparto posicional de [`build_page_seeds`]: cada heading `=`/`==`
/// ancla una página, de modo que el heading ancla n-ésimo cae en la página n+1. Las
/// secciones `=` son los headings de nivel 1; la portada (`title:`) no es heading.
fn fixed_sections(source: &str) -> (usize, Vec<usize>) {
    let fixed: Vec<HeadingInfo> = parse_typ_headings(source)
        .into_iter()
        .filter(|h| h.level >= 1 && h.level <= 2)
        .collect();
    let sections = fixed
        .iter()
        .enumerate()
        .filter(|(_, h)| h.level == 1)
        .map(|(i, _)| i)
        .collect();
    (fixed.len(), sections)
}

/// Para cada sección `=` del `.typ`, el rango de páginas `[inicio, fin]` que cubre.
///
/// La página se computa como en [`build_page_seeds`] (n-ésimo heading ancla `=`/`==`
/// → página n+1): una sección llega hasta la página anterior a la siguiente `=`, de
/// modo que las diapositivas `==` de dentro caen en su rango. La portada (`title:`)
/// y las `==` previas a la primera `=` no forman sección.
pub(crate) fn section_page_ranges(typ_content: &str) -> Vec<(usize, usize)> {
    let (anchor_count, sections) = fixed_sections(typ_content);
    let mut ranges = Vec::with_capacity(sections.len());

    for (k, &pos) in sections.iter().enumerate() {
        let start = pos + 1;
        // La siguiente sección empieza en la página `next + 1`, así que esta
        // termina en la página `next`; la última llega al último heading ancla.
        let end = sections.get(k + 1).copied().unwrap_or(anchor_count);
        if end >= start {
            ranges.push((start, end));
        }
    }

    ranges
}

/// Reparte B bloques de guion entre S secciones `=`, de forma monótona y cubriendo
/// todas las secciones.
///
/// Devuelve, por cada sección, el índice del primer bloque que le corresponde
/// (`None` nunca se emite: si hay bloques, toda sección recibe uno). Con `b == s`
/// es la identidad; con `b < s` se asigna a cada sección el bloque más cercano del
/// reparto proporcional; con `b > s` los bloques sobrantes se agregan a la misma
/// sección (el índice devuelto es el primero de su tramo). `b == 0` o `s == 0`
/// devuelve vacío.
pub(crate) fn map_blocks_to_sections(blocks: usize, sections: usize) -> Vec<Option<usize>> {
    if blocks == 0 || sections == 0 {
        return Vec::new();
    }
    if blocks == sections {
        return (0..sections).map(Some).collect();
    }

    (0..sections)
        .map(|section| {
            let index = if blocks < sections {
                // Redondeo del reparto proporcional: i·(b−1)/(s−1).
                let numerator = section * (blocks - 1) * 2 + (sections - 1);
                numerator / (2 * (sections - 1))
            } else {
                section * blocks / sections
            };
            Some(index.min(blocks - 1))
        })
        .collect()
}

/// Convierte un guion de grano grueso en bloques embebidos, mapeando sus bloques a
/// las secciones `=` del `.typ` por rango de páginas.
///
/// El Markdown de cada sección se escribe **replicado** tras cada heading del rango
/// de páginas de la sección (ver [`section_page_ranges`]), reutilizando el export
/// idempotente: reejecutar actualiza los bloques en lugar de duplicarlos. Devuelve
/// el `.typ` resultante y cuántos bloques del guion se escribieron.
pub(crate) fn embed_script_ranges_into_typ(script: &str, typ_source: &str) -> (String, usize) {
    let blocks = script_timeline_blocks(script);
    if blocks.is_empty() {
        return (typ_source.to_string(), 0);
    }

    let ranges = section_page_ranges(typ_source);
    if ranges.is_empty() {
        warn!(
            "el guion tiene {} bloques por tramos pero el documento no tiene ninguna sección `=` \
             donde anclarlos; no se escribe ningún bloque",
            blocks.len()
        );
        return (typ_source.to_string(), 0);
    }

    // El mapeo puro es la única fuente de asignación: sus índices delimitan los
    // bloques de cada sección (con `b > s` agrega los sobrantes a la misma
    // sección) y con `b < s` cada sección toma un único bloque, el más cercano.
    let mapping = map_blocks_to_sections(blocks.len(), ranges.len());
    let blocks_of = |section: usize| -> &[String] {
        let start = mapping[section].unwrap_or(0);
        let end = if blocks.len() >= ranges.len() {
            mapping
                .get(section + 1)
                .copied()
                .flatten()
                .unwrap_or(blocks.len())
        } else {
            start + 1
        };
        &blocks[start..end.min(blocks.len())]
    };

    let mut records = Vec::new();
    for (section, &(start, end)) in ranges.iter().enumerate() {
        let markdown = blocks_of(section)
            .iter()
            .filter(|body| !body.trim().is_empty())
            .map(String::as_str)
            .collect::<Vec<&str>>()
            .join("\n\n");
        if markdown.trim().is_empty() {
            continue;
        }
        for page in start..=end {
            records.push(NoteRecord {
                page_number: page,
                heading: None,
                content: markdown.clone(),
                source_hash: None,
                edited: false,
            });
        }
    }

    let written = blocks.iter().filter(|body| !body.trim().is_empty()).count();
    (render_exported_typ(typ_source, &records), written)
}

/// Convierte un guion Markdown en bloques embebidos dentro de `typ_source`.
///
/// Con secciones `## …`, empareja la n-ésima sección con el n-ésimo heading de
/// slide (`=`/`==`), en orden, reutilizando el export idempotente. Sin secciones
/// `##` pero con bloques por tramos de tiempo, aplica el **mapeo por rango** a las
/// secciones `=` (ver [`embed_script_ranges_into_typ`]). Sin ninguna de las dos
/// cosas no escribe nada y registra un aviso. Devuelve el `.typ` resultante y
/// cuántos bloques se escribieron.
pub(crate) fn embed_script_into_typ(script: &str, typ_source: &str) -> (String, usize) {
    let bodies = script_section_bodies(script);

    if bodies.is_empty() {
        if script_timeline_blocks(script).is_empty() {
            warn!(
                "el guion no tiene secciones `## …` ni bloques por tramos de tiempo; no se \
                 escribe ningún bloque"
            );
            return (typ_source.to_string(), 0);
        }
        return embed_script_ranges_into_typ(script, typ_source);
    }

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

    // --- Mapeo por rango de guiones de grano grueso -------------------------

    /// Guion de grano grueso con la **estructura real** de las escaletas: la
    /// cabecera ocupa su propia región entre dos separadores y el cuerpo llega en
    /// la región siguiente (no en la misma región que la cabecera).
    const GUION_TRAMOS: &str = "\
# Episodio 812 — Wave Terminal

Preámbulo del episodio que no es ningún bloque.

──────────────────────────────
0:00 – 2:00  🎬 INTRO
──────────────────────────────

📌 Puntos a tratar:
▪ Saludo del episodio.

🎯 Objetivo: presentar el episodio.

──────────────────────────────
2:00 – 6:30  📋 BLOQUE 1 — ¿Por qué systemd timers?
──────────────────────────────

Aquí va el **contenido** del bloque.

──────────────────────────────
6:30 – 12:00  📋 BLOQUE 2 — Wave Terminal
──────────────────────────────

Segundo bloque.
";

    /// `.typ` con portada, una `==` previa a la primera `=`, tres secciones `=`
    /// y diapositivas `==` dentro de cada rango.
    const TYP_SECCIONES: &str = "\
#set document(title: \"Episodio 812\")

== Previa
sin sección propia

= Intro
intro

== Detalle intro
detalle

= Bloque 1
uno

== Detalle bloque
detalle

= Cierre
fin
";

    /// `.typ` de dos secciones `=`, cada una cubriendo dos páginas.
    const TYP_RANGO: &str = "\
= Intro
contenido intro

== Detalle intro
detalle

= Bloque 1
contenido uno

== Detalle bloque
detalle
";

    /// Guion de dos tramos con preámbulo, en la estructura real de las escaletas.
    const GUION_DOS_TRAMOS: &str = "\
# Episodio

preámbulo

──────────
0:00 – 2:00  🎬 INTRO
──────────

Cuerpo del intro.

──────────
2:00 – 6:30  📋 BLOQUE 1
──────────

Cuerpo del bloque.
";

    #[test]
    fn bloques_por_tramos_basico() {
        let blocks = script_timeline_blocks(GUION_TRAMOS);

        assert_eq!(
            blocks.len(),
            3,
            "un bloque por cabecera con marca de tiempo"
        );
        assert!(
            blocks[0].starts_with("0:00 – 2:00  🎬 INTRO"),
            "{:?}",
            blocks[0]
        );
        assert!(
            blocks[0].contains("📌 Puntos a tratar:"),
            "el cuerpo de la región posterior va en el bloque: {:?}",
            blocks[0]
        );
        assert!(blocks[0].contains("▪ Saludo del episodio."));
        assert!(blocks[0].contains("🎯 Objetivo: presentar el episodio."));
        assert!(!blocks[0].contains("Preámbulo del episodio"));
        assert!(blocks[1].starts_with("2:00 – 6:30  📋 BLOQUE 1"));
        assert!(blocks[1].contains("**contenido**"));
        assert!(blocks[2].starts_with("6:30 – 12:00  📋 BLOQUE 2"));
        assert!(blocks[2].ends_with("Segundo bloque."));
    }

    #[test]
    fn cuerpo_tras_la_region_cabecera() {
        // Estructura real reportada en el episodio 812:
        // sep / cabecera / sep / cuerpo / sep / cabecera / sep / cuerpo.
        let md = "\
──────────
0:00 – 2:00  🎬 INTRO
──────────

📌 Puntos a tratar:
▪ Saludo.

──────────
2:00 – 6:30  📋 BLOQUE 1
──────────

🎯 Objetivo: explicar Wave.
";

        let blocks = script_timeline_blocks(md);

        assert_eq!(blocks.len(), 2, "nº de bloques = nº de cabeceras");
        assert_eq!(
            blocks[0],
            "0:00 – 2:00  🎬 INTRO\n\n📌 Puntos a tratar:\n▪ Saludo."
        );
        assert_eq!(
            blocks[1],
            "2:00 – 6:30  📋 BLOQUE 1\n\n🎯 Objetivo: explicar Wave."
        );
    }

    #[test]
    fn bloques_ignoran_preambulo_y_cola() {
        let md = "\
# Título del episodio

Preámbulo.

──────────
0:00 – 2:00  🎬 INTRO
──────────

Cuerpo del intro.

──────────
⏱ Duración total: 45:00 – 50:00
";

        let blocks = script_timeline_blocks(md);

        assert_eq!(blocks.len(), 1);
        assert_eq!(blocks[0], "0:00 – 2:00  🎬 INTRO\n\nCuerpo del intro.");
    }

    #[test]
    fn sin_tramos_devuelve_vacio() {
        let md = "# Título\n\n## Sección\n\nsin separadores de tramo\n";
        assert!(script_timeline_blocks(md).is_empty());
    }

    #[test]
    fn cola_con_rango_no_es_bloque() {
        // La cola menciona horas, pero no ABRE con la cabecera: no es un bloque.
        let md = "\
──────────
0:00 – 2:00  🎬 INTRO
Cuerpo intro.
──────────
Notas de cierre: duración total 45:00 – 50:00
";

        let blocks = script_timeline_blocks(md);

        assert_eq!(
            blocks.len(),
            1,
            "solo la región que abre con la cabecera: {blocks:?}"
        );
        assert_eq!(blocks[0], "0:00 – 2:00  🎬 INTRO\nCuerpo intro.");
    }

    #[test]
    fn rangos_por_seccion() {
        assert_eq!(
            section_page_ranges(TYP_SECCIONES),
            vec![(2, 3), (4, 5), (6, 6)]
        );
    }

    #[test]
    fn rangos_ignoran_portada() {
        let ranges = section_page_ranges(TYP_SECCIONES);
        assert_eq!(
            ranges.len(),
            3,
            "la portada y la `==` previa no son secciones"
        );
        assert_eq!(
            ranges[0].0, 2,
            "la primera sección arranca tras la `==` previa"
        );
    }

    #[test]
    fn mapeo_identidad() {
        assert_eq!(
            map_blocks_to_sections(3, 3),
            vec![Some(0), Some(1), Some(2)]
        );
        assert_eq!(map_blocks_to_sections(1, 1), vec![Some(0)]);
    }

    #[test]
    fn mapeo_b_menor_s() {
        // Reparto monótono que cubre todas las secciones con el bloque más cercano.
        assert_eq!(
            map_blocks_to_sections(2, 5),
            vec![Some(0), Some(0), Some(1), Some(1), Some(1)]
        );
        // Con un solo bloque, todas las secciones reciben el mismo (el más cercano).
        assert_eq!(
            map_blocks_to_sections(1, 3),
            vec![Some(0), Some(0), Some(0)]
        );
    }

    #[test]
    fn mapeo_b_mayor_s() {
        // Los bloques sobrantes se agregan a la misma sección: cada sección
        // devuelve el primer bloque de su tramo.
        assert_eq!(map_blocks_to_sections(5, 2), vec![Some(0), Some(2)]);
        assert_eq!(map_blocks_to_sections(4, 1), vec![Some(0)]);
        assert_eq!(
            map_blocks_to_sections(7, 3),
            vec![Some(0), Some(2), Some(4)]
        );
    }

    #[test]
    fn mapeo_vacio() {
        assert!(map_blocks_to_sections(0, 3).is_empty());
        assert!(map_blocks_to_sections(3, 0).is_empty());
        assert!(map_blocks_to_sections(0, 0).is_empty());
    }

    #[test]
    fn embed_replica_en_rango() {
        let (out, written) = embed_script_ranges_into_typ(GUION_DOS_TRAMOS, TYP_RANGO);

        assert_eq!(written, 2);
        let blocks = parse_embedded_blocks(&out);
        assert_eq!(blocks.len(), 4, "un bloque por página del rango: {out}");

        let intro = "0:00 – 2:00  🎬 INTRO\n\nCuerpo del intro.";
        let bloque = "2:00 – 6:30  📋 BLOQUE 1\n\nCuerpo del bloque.";
        assert_eq!(blocks[0].heading, "Intro");
        assert_eq!(blocks[0].markdown, intro);
        assert_eq!(blocks[1].heading, "Detalle intro");
        assert_eq!(
            blocks[1].markdown, intro,
            "replicado en la 2ª página del rango"
        );
        assert_eq!(blocks[2].heading, "Bloque 1");
        assert_eq!(blocks[2].markdown, bloque);
        assert_eq!(blocks[3].heading, "Detalle bloque");
        assert_eq!(blocks[3].markdown, bloque);
    }

    #[test]
    fn embed_rango_idempotente() {
        let (out1, _) = embed_script_ranges_into_typ(GUION_DOS_TRAMOS, TYP_RANGO);
        let (out2, _) = embed_script_ranges_into_typ(GUION_DOS_TRAMOS, &out1);

        assert_eq!(out1, out2);
        assert_eq!(out1.matches(">>> typshow:notes:end").count(), 4);
        assert_eq!(parse_embedded_blocks(&out2).len(), 4);
    }

    #[test]
    fn embed_agrega_bloques_sobrantes() {
        // 3 bloques para 2 secciones: los sobrantes se agregan a la misma sección.
        let script = "\
──────────
0:00 – 2:00  🎬 INTRO
──────────

Cuerpo del intro.

──────────
2:00 – 6:30  📋 BLOQUE 1
──────────

Cuerpo uno.

──────────
6:30 – 9:00  📋 BLOQUE 2
──────────

Cuerpo dos.
";

        let (out, written) = embed_script_ranges_into_typ(script, TYP_RANGO);

        assert_eq!(written, 3);
        let blocks = parse_embedded_blocks(&out);
        assert_eq!(
            blocks.len(),
            4,
            "2 secciones × 2 páginas ⇒ 4 réplicas: {out}"
        );
        assert_eq!(
            blocks[0].markdown,
            "0:00 – 2:00  🎬 INTRO\n\nCuerpo del intro."
        );
        let agregado =
            "2:00 – 6:30  📋 BLOQUE 1\n\nCuerpo uno.\n\n6:30 – 9:00  📋 BLOQUE 2\n\nCuerpo dos.";
        assert_eq!(blocks[2].markdown, agregado);
        assert_eq!(blocks[3].markdown, agregado);
    }

    #[test]
    fn embed_sin_secciones_no_escribe() {
        // Documento sin ninguna sección `=` (solo portada y una `==`).
        let typ = "#set document(title: \"X\")\n\n== Suelta\ncontenido\n";

        let (out, written) = embed_script_ranges_into_typ(GUION_DOS_TRAMOS, typ);

        assert_eq!(written, 0);
        assert_eq!(out, typ, "sin secciones `=` el .typ queda intacto");
    }

    /// Escribe `script` y `typ` en ficheros temporales y devuelve sus rutas.
    fn temp_script_and_typ(tag: &str, script: &str, typ: &str) -> (PathBuf, PathBuf) {
        let dir = std::env::temp_dir();
        let unique = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .map(|d| d.as_nanos())
            .unwrap_or(0);
        let script_path = dir.join(format!("typshow-{tag}-{}-{unique}.md", std::process::id()));
        let typ_path = dir.join(format!("typshow-{tag}-{}-{unique}.typ", std::process::id()));
        std::fs::write(&script_path, script).unwrap();
        std::fs::write(&typ_path, typ).unwrap();
        (script_path, typ_path)
    }

    // --- Detección de secciones consciente de fences e indentación ----------

    #[test]
    fn seccion_dentro_de_fence_ignorada() {
        // Escaleta real (episodio 807): el Markdown embebido en un fence contiene
        // líneas `## …` que NO son secciones del guion.
        let md = "\
## Real

cuerpo real

```markdown
## Falsa
contenido de ejemplo
```
";

        let bodies = script_section_bodies(md);

        assert_eq!(bodies.len(), 1, "solo cuenta la sección real: {bodies:?}");
        assert!(
            bodies[0].starts_with("cuerpo real\n\n```markdown"),
            "el fence pertenece al cuerpo de la sección real: {:?}",
            bodies[0]
        );
        assert!(
            !bodies[0].starts_with("## Falsa"),
            "`## Falsa` no abre una sección propia"
        );
    }

    #[test]
    fn cabecera_indentada_ignorada() {
        // 4 espacios = bloque de código indentado en Markdown, no cabecera.
        let md = "## Real\n\ncuerpo real\n\n    ## Falsa\n    contenido indentado\n";

        let bodies = script_section_bodies(md);

        assert_eq!(bodies.len(), 1, "la indentada no es sección: {bodies:?}");
        assert_eq!(
            bodies[0],
            "cuerpo real\n\n    ## Falsa\n    contenido indentado"
        );
    }

    #[test]
    fn secciones_reales_intactas() {
        let md = "## Uno\n\ncuerpo uno\n\n## Dos\n\ncuerpo dos\n\n## Tres\n\ncuerpo tres\n";

        let bodies = script_section_bodies(md);

        assert_eq!(bodies, vec!["cuerpo uno", "cuerpo dos", "cuerpo tres"]);
    }

    #[test]
    fn fences_mas_largos_y_tildes() {
        // Un fence de 4 backticks no se cierra con uno de 3, y `~~~` es fence.
        let md = "\
## Real

````text
```
## Aún dentro
```

## Falsa en tildes

~~~
## Otra dentro
~~~
";

        let bodies = script_section_bodies(md);

        assert_eq!(bodies.len(), 1, "solo la sección real: {bodies:?}");
        assert!(bodies[0].contains("## Falsa en tildes"));
    }

    #[test]
    fn tramos_con_markdown_embebido_usa_rango() {
        // Estructura real de escaleta: sep / cabecera / sep / cuerpo, con Markdown
        // embebido en un fence que contiene `## …`.
        let script = "\
──────────
0:00 – 2:00  🎬 INTRO
──────────

📌 Puntos a tratar:

```markdown
## Cómo usarla
comando de ejemplo
```

──────────
2:00 – 6:30  📋 BLOQUE 1
──────────

🎯 Objetivo: explicar el bloque.
";

        assert!(
            script_section_bodies(script).is_empty(),
            "el `##` dentro del fence no debe contar como sección"
        );

        let (_, written) = embed_script_into_typ(script, TYP_RANGO);

        assert_eq!(
            written, 2,
            "usa el modo rango: nº de bloques = nº de cabeceras, no 2 secciones `##`"
        );
    }

    #[test]
    fn cli_fallback_tramos() {
        let (script_path, typ_path) = temp_script_and_typ("rango", GUION_DOS_TRAMOS, TYP_RANGO);

        let written =
            guion_embed_files(&script_path.to_string_lossy(), &typ_path.to_string_lossy()).unwrap();

        assert_eq!(written, 2);
        let typ = std::fs::read_to_string(&typ_path).unwrap();
        assert_eq!(parse_embedded_blocks(&typ).len(), 4);
        assert!(typ.contains("// Cuerpo del bloque."));
        std::fs::remove_file(&script_path).ok();
        std::fs::remove_file(&typ_path).ok();
    }

    #[test]
    fn cli_usa_secciones_cuando_existen() {
        let script = "## Uno\n\ncuerpo uno\n\n## Dos\n\ncuerpo dos\n";
        let (script_path, typ_path) =
            temp_script_and_typ("secciones", script, "== Uno\nslide\n\n== Dos\nslide\n");

        let written =
            guion_embed_files(&script_path.to_string_lossy(), &typ_path.to_string_lossy()).unwrap();

        assert_eq!(written, 2, "con `##` se mantiene el mapeo 1:1");
        let typ = std::fs::read_to_string(&typ_path).unwrap();
        assert_eq!(parse_embedded_blocks(&typ).len(), 2);
        std::fs::remove_file(&script_path).ok();
        std::fs::remove_file(&typ_path).ok();
    }

    #[test]
    fn cli_sin_bloques_avisa() {
        let (script_path, typ_path) =
            temp_script_and_typ("vacio", "# Título\n\nsolo texto sin bloques\n", TYP_RANGO);

        let written =
            guion_embed_files(&script_path.to_string_lossy(), &typ_path.to_string_lossy()).unwrap();

        assert_eq!(written, 0);
        let typ = std::fs::read_to_string(&typ_path).unwrap();
        assert_eq!(typ, TYP_RANGO, "el .typ queda intacto");
        std::fs::remove_file(&script_path).ok();
        std::fs::remove_file(&typ_path).ok();
    }
}
