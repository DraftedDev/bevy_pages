use crate::page::Page;
use crate::parser::parse_page;
use crate::widgets::Widget;
use bevy::asset::io::Reader;
use bevy::asset::{AssetLoader, AsyncReadExt, LoadContext};
use bevy::prelude::TypePath;
use bevy::tasks::ConditionalSendFuture;
use roxmltree::{Document, ParsingOptions};
use rustc_hash::FxHashMap;
use std::error::Error;
use std::fmt::{Display, Formatter};

/// The loader for [Page] assets.
#[derive(Default, TypePath)]
pub struct PageLoader {
    pub(crate) initial_read_capacity: usize,
    pub(crate) widgets: FxHashMap<&'static str, Box<dyn Widget>>,
}

impl PageLoader {
    /// Retrieves a widget by its name or [None] if it isn't registered.
    #[inline(always)]
    pub fn get_widget(&self, name: &str) -> Option<&dyn Widget> {
        self.widgets.get(name).map(|w| w.as_ref())
    }
}

impl AssetLoader for PageLoader {
    type Asset = Page;
    type Settings = ();
    type Error = PageLoaderError;

    fn load(
        &self,
        reader: &mut dyn Reader,
        _: &Self::Settings,
        _: &mut LoadContext,
    ) -> impl ConditionalSendFuture<Output = Result<Self::Asset, Self::Error>> {
        Box::pin(async move {
            let mut string = String::with_capacity(self.initial_read_capacity);

            reader
                .read_to_string(&mut string)
                .await
                .map_err(PageLoaderError::Io)?;

            let doc = Document::parse_with_options(
                &string,
                ParsingOptions {
                    allow_dtd: true,
                    nodes_limit: u32::MAX,
                    entity_resolver: None,
                },
            )
            .map_err(PageLoaderError::Xml)?;

            let page = parse_page(self, doc).map_err(PageLoaderError::Parse)?;

            Ok(page)
        })
    }

    fn extensions(&self) -> &[&str] {
        &["xml"]
    }
}

/// The error type for [PageLoader].
#[derive(Debug)]
pub enum PageLoaderError {
    /// An IO-related error.
    Io(std::io::Error),
    /// An XML-related error.
    Xml(roxmltree::Error),
    /// A parse error.
    Parse(String),
}

impl Display for PageLoaderError {
    #[cold]
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            PageLoaderError::Io(e) => write!(f, "IO Error: {e}"),
            PageLoaderError::Xml(e) => write!(f, "XML Error: {e}"),
            PageLoaderError::Parse(e) => write!(f, "Parse Error: {e}"),
        }
    }
}

impl Error for PageLoaderError {}
