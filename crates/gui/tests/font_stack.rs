use std::sync::Arc;

use egui::epaint::{
    text::{FontData, FontDefinitions, FontFamily, FontId, FontTweak, FontsImpl},
    AlphaFromCoverage, Color32, Fonts, TextureAtlas,
};

fn custom_font(bytes: &'static [u8], index: u32) -> Fonts {
    let mut definitions = FontDefinitions::empty();
    let mut data = FontData::from_static(bytes);
    data.index = index;
    definitions
        .font_data
        .insert("custom".to_owned(), Arc::new(data));
    definitions
        .families
        .insert(FontFamily::Proportional, vec!["custom".to_owned()]);

    Fonts::new(2_048, AlphaFromCoverage::default(), definitions)
}

fn assert_font_renders(bytes: &'static [u8], index: u32, text: &str) {
    for pixels_per_point in [1.0, 1.5] {
        let mut fonts = custom_font(bytes, index);
        let font_id = FontId::proportional(24.0);
        assert!(
            fonts.has_glyphs(&font_id, text),
            "face {index} must contain every glyph in {text:?}"
        );

        let galley = fonts
            .with_pixels_per_point(pixels_per_point)
            .layout_no_wrap(text.to_owned(), font_id, Color32::WHITE);
        assert_eq!(galley.rows.len(), 1);
        assert_eq!(galley.rows[0].text(), text);
        assert!(galley.size().x.is_finite() && galley.size().x > 0.0);
        assert!(galley.size().y.is_finite() && galley.size().y > 0.0);
        assert!(galley.num_vertices > 0);
        assert!(galley.rows[0].row.glyphs[0].font_impl_ascent.is_finite());

        let delta = fonts
            .font_image_delta()
            .expect("rendering must dirty the font atlas");
        assert!(delta.image.size().into_iter().all(|side| side > 0));
    }
}

#[test]
fn default_fonts_render_at_one_x_and_fractional_scale() {
    for pixels_per_point in [1.0, 1.5] {
        let mut fonts = Fonts::new(
            2_048,
            AlphaFromCoverage::default(),
            FontDefinitions::default(),
        );
        let galley = fonts
            .with_pixels_per_point(pixels_per_point)
            .layout_no_wrap(
                "envctl 5090".to_owned(),
                FontId::proportional(18.0),
                Color32::WHITE,
            );

        assert_eq!(galley.rows[0].text(), "envctl 5090");
        assert!(galley.size().x > 0.0);
        assert!(galley.size().y > 0.0);
        assert!(fonts.font_image_delta().is_some());
    }
}

#[test]
fn custom_ttf_otf_and_ttc_faces_render() {
    assert_font_renders(font_test_data::TINOS_SUBSET, 0, "ABab");
    assert_font_renders(font_test_data::NOTO_SANS_JP_CFF, 0, "ABC");
    assert_font_renders(font_test_data::ttc::TTC, 0, "…");
    assert_font_renders(font_test_data::ttc::TTC, 1, "…");
}

#[test]
fn stock_epaint_033_font_api_remains_source_compatible() {
    let tweak = FontTweak {
        scale: 1.0,
        y_offset_factor: 0.0,
        y_offset: 0.0,
    };
    assert_eq!(tweak, FontTweak::default());

    let alpha = AlphaFromCoverage::default();
    let mut fonts = Fonts::new(1_024, alpha, FontDefinitions::default());
    fonts.begin_pass(2_048, alpha);
    assert_eq!(fonts.max_texture_side(), 2_048);
    assert_eq!(fonts.with_pixels_per_point(1.0).max_texture_side(), 2_048);

    let _fonts_impl = FontsImpl::new(1_024, alpha, FontDefinitions::default());
    let atlas = TextureAtlas::new([1_024, 32], alpha);
    assert_eq!(atlas.size(), [1_024, 32]);
}
