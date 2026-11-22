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
