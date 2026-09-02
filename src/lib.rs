#![doc = include_str!("../README.md")]
#![warn(missing_docs)]
#![allow(clippy::type_complexity)]
#![allow(clippy::too_many_arguments)]

use crate::manager::PageManager;
use crate::page::Page;
use crate::systems::PageSystemSet;
use crate::widgets::Widget;
use bevy::app::{App, Plugin, Update};
use bevy::asset::{AssetApp, AssetEvent};
use bevy::prelude::{IntoScheduleConfigs, Res, on_message};
use rustc_hash::FxHashMap;

/// Contains the basic element structures for a clean UI.
pub mod element;

/// Contains UI events and event handling systems.
pub mod events;

/// Contains the page asset loader.
pub mod loader;

/// Contains the [Page] struct for core UI work.
pub mod page;

/// Contains the [PageManager] resource and related systems to manage UI pages.
pub mod manager;

/// Contains custom widget functionality.
pub mod widgets;

/// Contains XML parsing functionality.
pub mod parser;

/// Contains the [Properties](props::Properties) struct.
pub mod props;

/// Contains utility functionality.
pub mod utils;

pub(crate) mod systems;

/// The main plugin for `bevy_pages`.
///
/// This is required to spawn and manage UI pages.
pub struct PagesPlugin {
    initial_read_capacity: usize,
    widgets: FxHashMap<&'static str, Box<dyn Widget>>,
}

impl PagesPlugin {
    /// Creates a new [PagesPlugin] instance with an empty widget registry.
    ///
    /// You should call [PagesPlugin::with_default_widgets] or [PagesPlugin::with_widget] to add widgets.
    ///
    /// Alternatively, you can use [PagesPlugin::default] which will add the default widgets automatically.
    #[inline(always)]
    pub fn empty() -> Self {
        Self {
            initial_read_capacity: 2048,
            widgets: FxHashMap::default(),
        }
    }

    /// Sets the initial read capacity for the asset loading process.
    ///
    /// This should be the average size (in bytes) of the XML files you plan to load.
    ///
    /// Defaults to `2048`.
    #[inline(always)]
    pub fn with_initial_read_capacity(mut self, initial_read_capacity: usize) -> Self {
        self.initial_read_capacity = initial_read_capacity;
        self
    }

    /// Registers a new widget to be parsed and spawned from XML.
    #[inline(always)]
    pub fn with_widget<W: Widget + Default>(mut self) -> Self {
        self.widgets.insert(W::name(), Box::new(W::default()));
        self
    }

    /// Registers the default widgets found in the [widgets] module.
    #[inline(always)]
    pub fn with_default_widgets(self) -> Self {
        self.with_widget::<widgets::node::NodeWidget>()
            .with_widget::<widgets::button::ButtonWidget>()
            .with_widget::<widgets::text::TextWidget>()
            .with_widget::<widgets::checkbox::CheckboxWidget>()
            .with_widget::<widgets::switch::SwitchWidget>()
            .with_widget::<widgets::text_input::TextInputWidget>()
            .with_widget::<widgets::slider::SliderWidget>()
            .with_widget::<widgets::progress_bar::ProgressBarWidget>()
            .with_widget::<widgets::image::ImageWidget>()
            .with_widget::<widgets::tooltip::TooltipWidget>()
            .with_widget::<widgets::dropdown::DropdownWidget>()
            .with_widget::<widgets::divider::DividerWidget>()
            .with_widget::<widgets::scroll_view::ScrollViewWidget>()
    }
}

impl Plugin for PagesPlugin {
    fn build(&self, app: &mut App) {
        app.init_asset::<Page>()
            .register_asset_loader(loader::PageLoader {
                initial_read_capacity: self.initial_read_capacity,
                widgets: self.widgets.clone(),
            })
            .insert_resource(PageManager::new())
            .configure_sets(
                Update,
                PageSystemSet.run_if(|manager: Res<PageManager>| manager.any_active()),
            )
            .add_systems(
                Update,
                (
                    manager::spawn_page.run_if(on_message::<AssetEvent<Page>>),
                    manager::de_activate_pages
                        .run_if(|manager: Res<PageManager>| manager.has_active_requests()),
                    (systems::update_state, systems::interactions)
                        .chain()
                        .in_set(PageSystemSet),
                ),
            )
            .add_observer(manager::despawn_page);

        #[cfg(feature = "hot-reload")]
        app.add_systems(Update, systems::hot_reload);

        for widget in self.widgets.values() {
            widget.setup(app);
        }
    }
}

impl Default for PagesPlugin {
    #[inline(always)]
    fn default() -> Self {
        Self {
            initial_read_capacity: 2048,
            widgets: FxHashMap::default(),
        }
        .with_default_widgets()
    }
}
