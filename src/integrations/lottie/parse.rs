use super::asset::VelloLottie;
use crate::integrations::VectorLoaderError;
use bevy::prelude::*;
use std::sync::Arc;

/// Deserialize a Lottie file from bytes.
pub fn load_lottie_from_bytes(bytes: &[u8]) -> Result<VelloLottie, VectorLoaderError> {
    // Load Lottie JSON bytes with the Velato (bodymovin) parser
    let composition = velato::Composition::from_slice(bytes).map_err(VectorLoaderError::Velato)?;

    let width = composition.width as f32;
    let height = composition.height as f32;

    let asset = VelloLottie {
        composition: Arc::new(composition),
        local_transform_center: {
            let mut transform = Transform::default();
            transform.translation.x = width / 2.0;
            transform.translation.y = -height / 2.0;
            transform
        },
        width,
        height,
        alpha: 1.0,
    };

    Ok(asset)
}

/// Deserialize a Lottie file from a string slice.
pub fn load_lottie_from_str(json_str: &str) -> Result<VelloLottie, VectorLoaderError> {
    let bytes = json_str.as_bytes();

    load_lottie_from_bytes(bytes)
}
