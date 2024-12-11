use crate::{
    integrations::{lottie::load_lottie_from_bytes, VectorLoaderError},
    VelloAsset,
};
use bevy::{
    asset::{io::Reader, AssetLoader, LoadContext},
    prelude::*,
    utils::ConditionalSendFuture,
};

#[derive(Default)]
pub struct VelloLottieLoader;

impl AssetLoader for VelloLottieLoader {
    type Asset = VelloAsset;

    type Settings = ();

    type Error = VectorLoaderError;

    fn load(
        &self,
        reader: &mut dyn Reader,
        _settings: &Self::Settings,
        load_context: &mut LoadContext,
    ) -> impl ConditionalSendFuture<Output = Result<Self::Asset, Self::Error>> {
        Box::pin(async move {
            let mut bytes = Vec::new();
            reader.read_to_end(&mut bytes).await?;
            let path = load_context.path().to_owned();
            let ext =
                path.extension()
                    .and_then(std::ffi::OsStr::to_str)
                    .ok_or(VectorLoaderError::Io(std::io::Error::new(
                        std::io::ErrorKind::InvalidData,
                        "Invalid file extension",
                    )))?;

            debug!("parsing {}...", load_context.path().display());
            match ext {
                "json" => {
                    let asset = load_lottie_from_bytes(&bytes)?;
                    info!(
                        path = format!("{}", load_context.path().display()),
                        size = format!("{:?}", (asset.width, asset.height)),
                        "finished parsing lottie json asset"
                    );
                    Ok(asset)
                }
                ext => Err(VectorLoaderError::Io(std::io::Error::new(
                    std::io::ErrorKind::InvalidData,
                    format!("Invalid file extension: '{ext}'"),
                ))),
            }
        })
    }

    fn extensions(&self) -> &[&str] {
        &["json"]
    }
}
