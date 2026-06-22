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
    assert!(pixmap.height() > 0, "Rendered pixmap should have height > 0");
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

#[test]
fn pdf_page_count_works() {
    // Create a minimal valid PDF
    let minimal_pdf = b"%PDF-1.4\n1 0 obj<</Type/Catalog/Pages 2 0 R>>endobj\n2 0 obj<</Type/Pages/Kids[3 0 R]/Count 1>>endobj\n3 0 obj<</Type/Page/Parent 2 0 R/MediaBox[0 0 612 792]>>endobj\nxref\n0 4\n0000000000 65535 f \n0000000009 00000 n \n0000000058 00000 n \n0000000115 00000 n \ntrailer<</Size 4/Root 1 0 R>>\nstartxref\n190\n%%EOF";
    let pdf = pdf_render::pdf_syntax::Pdf::new(minimal_pdf.to_vec());
    assert!(pdf.is_ok(), "PDF should parse correctly");
    let pdf = pdf.unwrap();
    assert_eq!(pdf.pages().len(), 1, "PDF should have 1 page");
}
