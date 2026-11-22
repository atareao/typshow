#[test]
fn typst_pipeline_works() {
    let content = r#"
#set page(paper: "a4", margin: 1cm)
= Test Presentation
This is a *test*.
"#;
    let world = typshow::TypstWorld::new(content, "/tmp/fake.typ");
    let result = typst::compile::<typst_layout::PagedDocument>(&world).output;
    assert!(result.is_ok(), "Typst compilation should succeed");
    let doc = result.unwrap();
    assert_eq!(doc.pages().len(), 1, "Should compile to 1 A4 page");
    let page = &doc.pages()[0];
    let pt_width = page.frame.size().x.to_pt();
    assert!(pt_width > 0.0, "Page should have positive width");

    let options = typst_render::RenderOptions {
        pixel_per_pt: typst::utils::Scalar::new(2.0),
        render_bleed: false,
    };
    let pixmap = typst_render::render(page, &options);
    assert!(pixmap.width() > 0, "Rendered pixmap should have width > 0");
    assert!(
        pixmap.height() > 0,
        "Rendered pixmap should have height > 0"
    );
}

#[test]
fn typst_pipeline_multi_page() {
    let content = r#"
#set page(paper: "a4", margin: 1cm)
= Slide 1
#pagebreak()
= Slide 2
#pagebreak()
= Slide 3
"#;
    let world = typshow::TypstWorld::new(content, "/tmp/fake.typ");
    let result = typst::compile::<typst_layout::PagedDocument>(&world).output;
    assert!(result.is_ok(), "Typst compilation should succeed");
    let doc = result.unwrap();
    assert_eq!(doc.pages().len(), 3, "Should have 3 pages");
}

/// El guion embebido son comentarios: no debe alterar en nada el PDF renderizado.
#[test]
fn guion_embebido_no_aparece_en_el_pdf() {
    let base = "== Uno\ncontenido uno\n";
    let with_notes = "== Uno\n\
                      // >>> typshow:notes\n\
                      // guion secreto de la página\n\
                      //\n\
                      // con **markdown**\n\
                      // >>> typshow:notes:end\n\
                      contenido uno\n";

    let render = |content: &str| {
        let world = typshow::TypstWorld::new(content, "/tmp/fake.typ");
        let doc = typst::compile::<typst_layout::PagedDocument>(&world)
            .output
            .expect("Typst compilation should succeed");
        let options = typst_render::RenderOptions {
            pixel_per_pt: typst::utils::Scalar::new(2.0),
            render_bleed: false,
        };
        (
            doc.pages().len(),
            typst_render::render(&doc.pages()[0], &options),
        )
    };

    let (pages_base, pixmap_base) = render(base);
    let (pages_notes, pixmap_notes) = render(with_notes);

    assert_eq!(pages_base, pages_notes, "el guion no añade páginas");
    assert_eq!(
        pixmap_base.data(),
        pixmap_notes.data(),
        "el guion embebido no debe cambiar un solo píxel del PDF"
    );
}

/// El guion embebido por **mapeo de rango** (guion de grano grueso, sin `##`)
/// tampoco debe alterar el PDF: son comentarios replicados en cada página del
/// rango de su sección `=`.
#[test]
fn guion_rango_no_aparece_en_el_pdf() {
    const SCRIPT: &str = "# Episodio\n\npreámbulo\n\n──────────\n0:00 – 2:00  🎬 INTRO\n──────────\n\nCuerpo del intro.\n\n──────────\n2:00 – 6:30  📋 BLOQUE 1\n──────────\n\nCuerpo del bloque.\n";
    const BASE: &str = "= Intro\ncontenido uno\n\n= Bloque 1\ncontenido dos\n";

    let dir = std::env::temp_dir();
    let unique = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map(|d| d.as_nanos())
        .unwrap_or(0);
    let script_path = dir.join(format!("typshow-rango-{}-{unique}.md", std::process::id()));
    let typ_path = dir.join(format!("typshow-rango-{}-{unique}.typ", std::process::id()));
    std::fs::write(&script_path, SCRIPT).expect("escribir el guion temporal");
    std::fs::write(&typ_path, BASE).expect("escribir el .typ temporal");

    let written =
        typshow::guion_embed_files(&script_path.to_string_lossy(), &typ_path.to_string_lossy())
            .expect("el mapeo por rango no debe fallar");
    assert_eq!(written, 2, "los dos bloques del guion se escriben");
    let with_notes = std::fs::read_to_string(&typ_path).expect("leer el .typ convertido");
    assert!(
        with_notes.contains("// >>> typshow:notes"),
        "el guion debe quedar embebido como comentario: {with_notes}"
    );

    let render = |content: &str| {
        let world = typshow::TypstWorld::new(content, "/tmp/fake.typ");
        let doc = typst::compile::<typst_layout::PagedDocument>(&world)
            .output
            .expect("Typst compilation should succeed");
        let options = typst_render::RenderOptions {
            pixel_per_pt: typst::utils::Scalar::new(2.0),
            render_bleed: false,
        };
        (
            doc.pages().len(),
            typst_render::render(&doc.pages()[0], &options),
        )
    };

    let (pages_base, pixmap_base) = render(BASE);
    let (pages_notes, pixmap_notes) = render(&with_notes);

    assert_eq!(pages_base, pages_notes, "el guion no añade páginas");
    assert_eq!(
        pixmap_base.data(),
        pixmap_notes.data(),
        "el guion embebido por rango no debe cambiar un solo píxel del PDF"
    );

    std::fs::remove_file(&script_path).ok();
    std::fs::remove_file(&typ_path).ok();
}
