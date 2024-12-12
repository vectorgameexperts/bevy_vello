use super::{vello_text::VelloTextSection, VelloTextAnchor};
use bevy::{prelude::*, reflect::TypePath, render::render_asset::RenderAsset};
use std::sync::Arc;
use vello::{
    kurbo::Affine,
    peniko::{self, Blob, Font},
    skrifa::{FontRef, MetadataProvider},
    Glyph, Scene,
};

const VARIATIONS: &[(&str, f32)] = &[];

#[derive(Asset, TypePath, Debug, Clone)]
pub struct VelloFont {
    pub font: peniko::Font,
}

impl RenderAsset for VelloFont {
    type SourceAsset = VelloFont;

    type Param = ();

    fn prepare_asset(
        source_asset: Self::SourceAsset,
        _param: &mut bevy::ecs::system::SystemParamItem<Self::Param>,
    ) -> Result<Self, bevy::render::render_asset::PrepareAssetError<Self::SourceAsset>> {
        Ok(source_asset)
    }
}

impl VelloFont {
    pub fn new(font_data: Vec<u8>) -> Self {
        Self {
            font: Font::new(Blob::new(Arc::new(font_data)), 0),
        }
    }

    pub fn sizeof(&self, text: &VelloTextSection) -> Vec2 {
        let font = FontRef::new(self.font.data.data()).expect("Vello font creation error");
        let font_size = vello::skrifa::instance::Size::new(text.style.font_size);
        let charmap = font.charmap();
        let axes = font.axes();
        // TODO: What do Variations here do? Any font nerds know? I'm definitely not doing this
        // right.
        let var_loc = axes.location(VARIATIONS);
        let metrics = font.metrics(font_size, &var_loc);
        let line_height = metrics.ascent - metrics.descent + metrics.leading;
        let glyph_metrics = font.glyph_metrics(font_size, &var_loc);

        // TODO: Parley recently implemented type hinting, I should be handling this.
        let mut pen_x = 0.0;
        let mut pen_y: f32 = 0.0;
        let mut width: f32 = 0.0;
        for ch in text.value.chars() {
            if ch == '\n' {
                pen_y += line_height;
                pen_x = 0.0;
                continue;
            }
            let gid = charmap.map(ch).unwrap_or_default();
            let advance = glyph_metrics.advance_width(gid).unwrap_or_default();

            pen_x += advance;
            width = width.max(pen_x);
        }
        let height: f32 = metrics.cap_height.unwrap_or(line_height) + pen_y;
        Vec2::new(width, height)
    }

    pub(crate) fn render(
        &self,
        scene: &mut Scene,
        mut transform: Affine,
        text: &VelloTextSection,
        text_anchor: VelloTextAnchor,
    ) {
        let font = FontRef::new(self.font.data.data()).expect("Vello font creation error");

        let font_size = vello::skrifa::instance::Size::new(text.style.font_size);
        let charmap = font.charmap();
        let axes = font.axes();
        let var_loc = axes.location(VARIATIONS);
        let metrics = font.metrics(font_size, &var_loc);
        let line_height = metrics.ascent - metrics.descent + metrics.leading;
        let glyph_metrics = font.glyph_metrics(font_size, &var_loc);

        let mut pen_x = 0f32;
        let mut pen_y = 0f32;
        let mut width = 0f32;
        let glyphs: Vec<Glyph> = text
            .value
            .chars()
            .filter_map(|ch| {
                if ch == '\n' {
                    pen_y += line_height;
                    pen_x = 0.0;
                    return None;
                }
                let gid = charmap.map(ch).unwrap_or_default();
                let advance = glyph_metrics.advance_width(gid).unwrap_or_default();
                let x = pen_x;
                pen_x += advance;
                width = width.max(pen_x);
                Some(Glyph {
                    id: gid.to_u32(),
                    x,
                    y: pen_y,
                })
            })
            .collect();
        // Push up from pen_y
        transform *= vello::kurbo::Affine::translate((0.0, -pen_y as f64));

        // Alignment settings
        let width = width as f64;
        let height = (metrics.cap_height.unwrap_or(line_height) + pen_y) as f64;
        match text_anchor {
            VelloTextAnchor::TopLeft => {
                transform *= vello::kurbo::Affine::translate((0.0, height));
            }
            VelloTextAnchor::Left => {
                transform *= vello::kurbo::Affine::translate((0.0, height / 2.0));
            }
            VelloTextAnchor::BottomLeft => {
                transform *= vello::kurbo::Affine::translate((0.0, 0.0));
            }
            VelloTextAnchor::Top => {
                transform *= vello::kurbo::Affine::translate((-width / 2.0, height));
            }
            VelloTextAnchor::Center => {
                transform *= vello::kurbo::Affine::translate((-width / 2.0, height / 2.0));
            }
            VelloTextAnchor::Bottom => {
                transform *= vello::kurbo::Affine::translate((-width / 2.0, 0.0));
            }
            VelloTextAnchor::TopRight => {
                transform *= vello::kurbo::Affine::translate((-width, height));
            }
            VelloTextAnchor::Right => {
                transform *= vello::kurbo::Affine::translate((-width, height / 2.0));
            }
            VelloTextAnchor::BottomRight => {
                transform *= vello::kurbo::Affine::translate((-width, 0.0));
            }
        }

        scene
            .draw_glyphs(&self.font)
            .font_size(text.style.font_size)
            .transform(transform)
            .normalized_coords(var_loc.coords())
            .brush(&text.style.brush.clone())
            .draw(vello::peniko::Fill::EvenOdd, glyphs.into_iter());
    }
}
