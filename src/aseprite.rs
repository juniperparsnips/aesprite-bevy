use core::str;
use std::{collections::HashMap, iter::FusedIterator};

use bevy::{
    asset::{io::Reader, Asset, AssetLoader, Handle, LoadContext},
    color::Color,
    image::Image,
    math::{URect, UVec2},
    reflect::TypePath,
    sprite::{TextureAtlas, TextureAtlasLayout},
};
use serde::Deserialize;
use thiserror::Error;

#[derive(Debug, Clone, Copy, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "lowercase")]
pub enum AnimationDirection {
    Forward,
    Reverse,
}

#[derive(Debug, Clone, Copy, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "lowercase")]
enum BlendMode {
    Normal,
}

#[derive(Debug, Clone, Deserialize)]
pub struct AsepriteLayer {
    name: String,
    opacity: u8,
    blend_mode: BlendMode,
}

#[derive(Debug, Clone)]
pub struct AsepriteState {
    pub name: String,
    pub direction: AnimationDirection,
    pub color: Color,
    pub atlas: TextureAtlas,
    /// Duration of a frame (ms)
    pub durations: Vec<usize>,
}

impl AsepriteState {
    fn new(
        tag: &FrameTag,
        aseprite_json: &AsepriteJson,
        load_context: &mut LoadContext<'_>,
    ) -> Result<Self, AsepriteError> {
        match aseprite_json.frames {
            AsepriteFrames::Dict(_) => {
                return Err(AsepriteError::Unsupported(
                    "Frames as dictionary".to_string(),
                ))
            },
            _ => {},
        }

        if tag.from > tag.to {
            return Err(AsepriteError::InvalidTagRange(tag.from, tag.to));
        }

        let mut durations = Vec::with_capacity(tag.to - tag.from + 1);
        let mut atlas_layout = TextureAtlasLayout::new_empty(aseprite_json.meta.size.into());
        for frame in aseprite_json.frames.slice(tag.from, tag.to, tag.direction) {
            if frame.rotated {
                return Err(AsepriteError::Unsupported("Frame Rotation".to_string()));
            }
            if frame.trimmed || frame.source_size != frame.frame.into() {
                return Err(AsepriteError::Unsupported("Sprite Trimming".to_string()));
            }
            if frame.frame != frame.sprite_source_size {
                return Err(AsepriteError::Unsupported("Cel Trimming".to_string()));
            }

            atlas_layout.add_texture(frame.frame.into());
            durations.push(frame.duration);
        }
        let layout_handle = load_context.add_labeled_asset(tag.name.clone(), atlas_layout);

        let atlas = TextureAtlas {
            layout: layout_handle,
            index: 0,
        };

        Ok(Self {
            name: tag.name.clone(),
            direction: tag.direction,
            color: tag.color,
            atlas,
            durations,
        })
    }
}

#[derive(Asset, TypePath, Debug)]
pub struct AsepriteAnimation {
    pub image: Handle<Image>,
    pub states: HashMap<String, AsepriteState>,
}

impl AsepriteAnimation {
    fn new(
        aseprite_json: AsepriteJson,
        load_context: &mut LoadContext<'_>,
    ) -> Result<Self, AsepriteError> {
        let mut states = HashMap::new();
        for tag in &aseprite_json.meta.frame_tags {
            states.insert(
                tag.name.clone(),
                AsepriteState::new(tag, &aseprite_json, load_context)?,
            );
        }

        let image = load_context.load::<Image>(aseprite_json.meta.image);

        Ok(Self { image, states })
    }
}

#[derive(Default)]
pub struct AsepriteLoader;

#[non_exhaustive]
#[derive(Debug, Error)]
pub enum AsepriteLoaderError {
    /// An [IO](std::io) Error
    #[error("Could not load asset: {0}")]
    Io(#[from] std::io::Error),
    /// A [Serde-JSON](serde_json) Error
    #[error("Could not parse JSON: {0}")]
    Json(#[from] serde_json::Error),
    /// The asprite JSON was parsed properly but still can't be used
    #[error(transparent)]
    Aseprite(#[from] AsepriteError),
}

#[derive(Debug, Error)]
pub enum AsepriteError {
    #[error("Unsupported aseprite feature used: {0}")]
    Unsupported(String),
    #[error("Invalid frame tag range (from {0} to {1})")]
    InvalidTagRange(usize, usize),
}

impl AssetLoader for AsepriteLoader {
    type Asset = AsepriteAnimation;
    type Settings = ();
    type Error = AsepriteLoaderError;
    async fn load(
        &self,
        reader: &mut dyn Reader,
        _settings: &(),
        load_context: &mut LoadContext<'_>,
    ) -> Result<Self::Asset, Self::Error> {
        let mut bytes = Vec::new();
        reader.read_to_end(&mut bytes).await?;

        let aseprite: AsepriteJson = serde_json::from_slice(&bytes)?;

        Ok(AsepriteAnimation::new(aseprite, load_context)?)
    }

    fn extensions(&self) -> &[&str] {
        &["json"]
    }
}

#[derive(Debug, Clone, Copy, Deserialize, PartialEq)]
struct AsepriteRect {
    x: u32,
    y: u32,
    w: u32,
    h: u32,
}

impl From<AsepriteRect> for URect {
    fn from(v: AsepriteRect) -> Self {
        Self {
            min: UVec2 { x: v.x, y: v.y },
            max: UVec2 {
                x: v.x + v.w,
                y: v.y + v.h,
            },
        }
    }
}

impl From<AsepriteRect> for AsepriteSize {
    fn from(v: AsepriteRect) -> Self {
        Self { w: v.w, h: v.h }
    }
}

#[derive(Debug, Clone, Copy, Deserialize, PartialEq)]
struct AsepriteSize {
    w: u32,
    h: u32,
}

impl From<AsepriteSize> for UVec2 {
    fn from(v: AsepriteSize) -> Self {
        Self { x: v.w, y: v.h }
    }
}

#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct AsepriteFrame {
    #[serde(rename = "filename")]
    _filename: String,
    frame: AsepriteRect,
    rotated: bool,
    trimmed: bool,
    sprite_source_size: AsepriteRect,
    source_size: AsepriteSize,
    /// Duration the frame is shown (ms)
    duration: usize,
}

#[derive(Debug, Clone, Deserialize)]
#[serde(untagged)]
enum AsepriteFrames {
    List(Vec<AsepriteFrame>),
    Dict(HashMap<String, AsepriteFrame>),
}

impl AsepriteFrames {
    fn get(&self, i: usize) -> Option<&AsepriteFrame> {
        match self {
            AsepriteFrames::List(l) => l.get(i),
            AsepriteFrames::Dict(_d) => {
                // Aseprite allows tons of different formats for the file names
                // [Docs here](https://www.aseprite.org/docs/cli/#filename-format)
                // It would be too much work to parse, especially since the user could choose
                // to omit the frame number / start from different numbers etc.
                todo!()
            },
        }
    }
}

impl AsepriteFrames {
    fn slice<'a, 'b>(
        &'a self,
        from: usize,
        to: usize,
        direction: AnimationDirection,
    ) -> FramesIter<'b>
    where
        'a: 'b,
    {
        let next_i = match direction {
            AnimationDirection::Forward => from,
            AnimationDirection::Reverse => to,
        };

        FramesIter {
            frames: self,
            direction,
            from,
            to,
            next_i,
        }
    }
}

struct FramesIter<'a> {
    frames: &'a AsepriteFrames,
    direction: AnimationDirection,
    from: usize,
    to: usize,
    next_i: usize,
}

impl<'a> FramesIter<'a> {
    fn next_index(&mut self) -> Option<usize> {
        match self.direction {
            AnimationDirection::Forward => {
                if self.next_i <= self.to {
                    let old = self.next_i;
                    self.next_i += 1;
                    Some(old)
                } else {
                    None
                }
            },
            AnimationDirection::Reverse => {
                if self.next_i >= self.from {
                    let old = self.next_i;
                    self.next_i -= 1;
                    Some(old)
                } else {
                    None
                }
            },
        }
    }
}

impl<'a> Iterator for FramesIter<'a> {
    type Item = &'a AsepriteFrame;

    fn next(&mut self) -> Option<Self::Item> {
        self.next_index().and_then(|i| self.frames.get(i))
    }
}

impl<'a> FusedIterator for FramesIter<'a> {}

#[derive(Debug, Clone, Deserialize)]
struct FrameTag {
    name: String,
    from: usize,
    to: usize,
    direction: AnimationDirection,
    color: Color, // todo!() parse sRGB from '#<RR><GG><BB><AA>'
}

#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
struct AsepriteMeta {
    app: String,
    version: String,
    image: String,
    format: String,
    size: AsepriteSize,
    scale: String,
    // aseprite allows omitting tags in addition to layers and slices but we'd have nothing to do w/o tags
    frame_tags: Vec<FrameTag>,
    layers: Option<Vec<AsepriteLayer>>,
    // unknown what this is as my example is empty
    slices: Option<Vec<()>>,
}

#[derive(Debug, Clone, Deserialize)]
struct AsepriteJson {
    frames: AsepriteFrames,
    meta: AsepriteMeta,
}
