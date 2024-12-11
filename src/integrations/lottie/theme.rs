//! A component to augment playback colors.
//!
//! A long-term vision here is a selector-styled language, but now is just color swapping by layer
//! name.

use bevy::{prelude::*, utils::HashMap};
use velato::{
    model::{Brush, Shape},
    Composition,
};

#[derive(PartialEq, Component, Default, Clone, Debug, Reflect)]
#[reflect(Component)]
/// Add this component to a `VelloAssetBundle` entity to enable runtime color
/// editing. This interface allows swapping colors in a lottie composition by
/// selecting the desired layer and shape and overriding the original color with
/// a new color.
///
/// Only works for layer shapes with fill or stroke elements.
pub struct Theme {
    pub(crate) colors: HashMap<String, Color>,
}

impl Theme {
    pub fn new() -> Self {
        Self {
            colors: HashMap::default(),
        }
    }

    /// Swap a color with the given layer name.
    pub fn add(mut self, layer_name: &str, color: Color) -> Self {
        self.colors.insert(layer_name.to_string(), color);
        self
    }

    /// Swap a color for the selected layer name. This will overwrite the
    /// previous value.
    pub fn edit(&mut self, layer_name: &str, color: Color) -> &mut Self {
        self.colors.insert(layer_name.to_string(), color);
        self
    }

    pub fn get(&self, layer_name: &str) -> Option<&Color> {
        self.colors.get(layer_name)
    }

    pub fn get_mut(&mut self, layer_name: &str) -> Option<&mut Color> {
        self.colors.get_mut(layer_name)
    }
}

impl Theme {
    pub fn recolor(&self, composition: &Composition) -> Composition {
        let mut composition = composition.clone();
        'layers: for layer in composition.layers.iter_mut() {
            // Continue if this layer doesn't have a color swap
            let Some(target_color) = self.colors.get(&layer.name) else {
                continue 'layers;
            };
            let shapes = match &mut layer.content {
                velato::model::Content::Shape(shapes) => shapes,
                velato::model::Content::None | velato::model::Content::Instance { .. } => {
                    continue 'layers;
                }
            };
            // TODO: Vello hasn't fully implemented color spaces yet, so I'm very unsure of
            // which color space to use here.
            let target_color = target_color.to_linear();
            let target_color = vello::peniko::Color::from_rgba8(
                target_color.red as _,
                target_color.green as _,
                target_color.blue as _,
                target_color.alpha as _,
            );
            for shape in shapes.iter_mut() {
                recolor_shape(shape, target_color);
            }
        }
        composition
    }
}

/// A helper method to recolor a shape with a target color.
fn recolor_shape(shape: &mut Shape, target_color: vello::peniko::Color) {
    match shape {
        velato::model::Shape::Group(shapes, _) => {
            for shape in shapes.iter_mut() {
                recolor_shape(shape, target_color);
            }
        }
        velato::model::Shape::Draw(draw) => {
            recolor_brush(&mut draw.brush, target_color);
        }
        velato::model::Shape::Repeater(_) | velato::model::Shape::Geometry(_) => {}
    }
}

/// A helper method to recolor a brush with a target color.
fn recolor_brush(brush: &mut Brush, target_color: vello::peniko::Color) {
    match brush {
        velato::model::Brush::Fixed(brush) => match brush {
            vello::peniko::Brush::Solid(solid) => {
                *solid = target_color;
            }
            vello::peniko::Brush::Gradient(gradient) => {
                for stop in gradient.stops.iter_mut() {
                    stop.color = target_color;
                }
            }
            vello::peniko::Brush::Image(_) => {}
        },
        velato::model::Brush::Animated(brush) => match brush {
            velato::model::animated::Brush::Solid(brush) => match brush {
                velato::model::Value::Fixed(solid) => {
                    *solid = target_color;
                }
                velato::model::Value::Animated(keyframes) => {
                    for solid in keyframes.values.iter_mut() {
                        *solid = target_color;
                    }
                }
            },
            velato::model::animated::Brush::Gradient(gr) => match &mut gr.stops {
                velato::model::ColorStops::Fixed(stops) => {
                    for stop in stops.iter_mut() {
                        stop.color = target_color;
                    }
                }
                velato::model::ColorStops::Animated(stops) => {
                    for _ in 0..stops.count {
                        for stop in stops.values.iter_mut() {
                            let _offset = stop[0];

                            let [cr, cg, cb, ca] = target_color.components;

                            let r = &mut stop[1];
                            *r = cr as f64;
                            let g = &mut stop[2];
                            *g = cg as f64;
                            let b = &mut stop[3];
                            *b = cb as f64;
                            let a = &mut stop[4];
                            *a = ca as f64;
                        }
                    }
                }
            },
        },
    }
}
