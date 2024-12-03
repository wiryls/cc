use anyhow::Ok;
use bevy::{
    asset::{io::Reader, Asset, AssetLoader, AsyncReadExt, LoadContext, LoadedAsset},
    reflect::TypePath,
};
use cube_core::seed::Seed;
use serde::Deserialize;

/////////////////////////////////////////////////////////////////////////////
// LevelSeed

#[derive(Asset, Clone, TypePath)]
pub struct LevelSeeds(pub Vec<Seed>);

/////////////////////////////////////////////////////////////////////////////
// Loader

#[derive(Deserialize)]
struct LevelIndex {
    pub directory: String,
    pub extension: String,
    pub name_list: Vec<String>,
}

#[derive(Default)]
pub struct SeedsAssetLoader;
impl AssetLoader for SeedsAssetLoader {
    type Asset = LevelSeeds;
    type Error = anyhow::Error;
    type Settings = ();

    async fn load(
        &self,
        reader: &mut dyn Reader,
        _: &Self::Settings,
        load_context: &mut LoadContext<'_>,
    ) -> Result<Self::Asset, Self::Error> {
        const LEVEL_MARK: &str = "map";
        const INDEX_MARK: &str = "name_list";

        use super::LevelSource;
        use toml::Value;

        let mut texts = String::new();
        reader.read_to_string(&mut texts).await?;
        let value = toml::from_str::<Value>(&texts)?;
        if let Value::Table(table) = &value {
            if table.contains_key(LEVEL_MARK) {
                // level
                let source = value.try_into::<LevelSource>()?;
                let target = source.into_seed()?;
                return Ok(LevelSeeds(vec![target]));
            } else if table.contains_key(INDEX_MARK) {
                // index
                let source = value.try_into::<LevelIndex>()?;
                let folder = std::path::Path::new(&source.directory);
                let mut output = LevelSeeds(vec![]);
                for name in source.name_list {
                    let path = folder.join([&name, ".", &source.extension].concat());
                    let load: LoadedAsset<LevelSeeds> =
                        load_context.loader().immediate().load(path).await?;
                    output.0.append(&mut load.take().0);
                }
                return Ok(output);
            }
        }
        anyhow::bail!(
            "invalid toml file {}",
            load_context.path().to_string_lossy()
        );
    }

    fn extensions(&self) -> &[&str] {
        &["toml"]
    }
}
