#![doc = include_str!("../README.md")]
#![warn(missing_docs)]
#![allow(clippy::type_complexity)]
#![allow(clippy::too_many_arguments)]

use crate::page::Page;
use crate::spawner::PageSpawner;
use crate::systems::PageSystemSet;
use crate::widgets::Widget;
use bevy::app::{App, Plugin, Update};
use bevy::asset::{AssetApp, AssetEvent};
use bevy::prelude::{IntoScheduleConfigs, on_message, resource_exists};
use rustc_hash::FxHashMap;

/// Contains the basic element structures for a clean UI.
pub mod element;

/// Contains UI events and event handling systems.
pub mod events;

/// Contains the page asset loader.
pub mod loader;

/// Contains the [Page] struct for core UI work.
pub mod page;

/// Contains the [PageSpawner] resource and related systems to spawn UI pages.
pub mod spawner;

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
            .insert_resource(PageSpawner::new())
            .configure_sets(Update, PageSystemSet.run_if(resource_exists::<Page>))
            // General Interactions
            .add_systems(Update, systems::interactions.in_set(PageSystemSet))
            // Text Widget logic
            .add_systems(Update, widgets::text::update_props.in_set(PageSystemSet))
            // Image Widget logic
            .add_systems(Update, widgets::image::update_props.in_set(PageSystemSet))
            // Checkbox Widget Logic
            .add_systems(
                Update,
                (
                    widgets::checkbox::update_props,
                    widgets::checkbox::sync_visuals,
                )
                    .in_set(PageSystemSet),
            )
            .add_observer(widgets::checkbox::toggle_checkbox)
            // Switch Widget Logic
            .add_systems(
                Update,
                (widgets::switch::sync_visuals, widgets::switch::update_props)
                    .in_set(PageSystemSet),
            )
            .add_observer(widgets::switch::toggle_switch)
            // Text Input Widget Logic
            .add_systems(
                Update,
                (
                    widgets::text_input::update_props,
                    widgets::text_input::sync_visuals,
                    widgets::text_input::handle_typing,
                    widgets::text_input::handle_click_outside,
                )
                    .in_set(PageSystemSet),
            )
            .add_observer(widgets::text_input::handle_focus)
            // Slider Widget Logic
            .add_systems(
                Update,
                (widgets::slider::update_props, widgets::slider::sync_visuals)
                    .in_set(PageSystemSet),
            )
            // Progress Bar Widget Logic
            .add_systems(
                Update,
                (
                    widgets::progress_bar::update_props,
                    widgets::progress_bar::sync_progress_bar_visuals,
                )
                    .in_set(PageSystemSet),
            )
            // Tooltip Widget Logic
            .add_systems(
                Update,
                (
                    widgets::tooltip::update_props,
                    widgets::tooltip::sync_visuals,
                )
                    .in_set(PageSystemSet),
            )
            // Scroll View Widget Logic
            .add_systems(
                Update,
                (
                    widgets::scroll_view::update_props,
                    widgets::scroll_view::update_scroll_bounds,
                    widgets::scroll_view::scroll_view_mouse_wheel,
                    widgets::scroll_view::scroll_view_keyboard,
                    widgets::scroll_view::apply_scroll_physics,
                    widgets::scroll_view::update_visuals,
                )
                    .chain()
                    .in_set(PageSystemSet),
            )
            // Dropdown Widget Logic
            .add_systems(
                Update,
                (
                    widgets::dropdown::update_props,
                    widgets::dropdown::option_select,
                    widgets::dropdown::trigger_menu,
                    widgets::dropdown::visibility,
                    widgets::dropdown::close_on_outside_click,
                )
                    .in_set(PageSystemSet),
            )
            // Spawn & Despawn Logic
            .add_systems(
                Update,
                spawner::spawn_page.run_if(on_message::<AssetEvent<Page>>),
            )
            .add_observer(spawner::despawn_page);
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
