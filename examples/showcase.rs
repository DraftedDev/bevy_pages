use bevy::DefaultPlugins;
use bevy::app::{App, PluginGroup, Startup};
use bevy::asset::{AssetPlugin, AssetServer};
use bevy::camera::{Camera, Camera2d};
use bevy::prelude::{Commands, On, Query, Res, ResMut, Single, Transform};
use bevy_pages::PagesPlugin;
use bevy_pages::events::{ElementClick, ElementSet, ElementToggle};
use bevy_pages::manager::PageManager;
use bevy_pages::props::Properties;
use bevy_pages::widgets::notifier::NotifyMessage;
use bevy_pages::widgets::progress_bar::ProgressBarProps;
use bevy_pages::widgets::text::TextProps;

fn main() {
    App::new()
        .add_plugins((
            DefaultPlugins.set(AssetPlugin {
                watch_for_changes_override: Some(cfg!(feature = "hot-reload")),
                ..Default::default()
            }),
            PagesPlugin::default(),
        ))
        .add_systems(Startup, setup)
        .add_observer(counter)
        .add_observer(checkbox)
        .add_observer(slider)
        .add_observer(switch)
        .run();
}

fn setup(mut commands: Commands, assets: Res<AssetServer>, mut manager: ResMut<PageManager>) {
    commands.spawn((Camera2d::default(), Camera::default(), Transform::default()));
    let handle = assets.load("showcase.xml");

    manager.spawn("showcase", handle);
    manager.set_active("showcase", true);
}

fn counter(
    click: On<ElementClick>,
    mut commands: Commands,
    manager: Res<PageManager>,
    mut query: Query<&mut Properties<TextProps>>,
) {
    if let Some(page) = manager.get("showcase") {
        let counter_entity = page.get("counter");
        let mut props = query.get_mut(counter_entity).unwrap();
        let counter = props.default.content.parse::<i32>().unwrap();

        if click.matches_id("increment") {
            commands.write_message(NotifyMessage::new(format!(
                "Counter set to {}",
                counter + 1
            )));

            props.mutate(|props| props.content = (counter + 1).to_string());
        }

        if click.matches_id("decrement") {
            commands.write_message(NotifyMessage::new(format!(
                "Counter set to {}",
                counter - 1
            )));

            props.mutate(|props| props.content = (counter - 1).to_string());
        }
    }
}

fn checkbox(
    toggle: On<ElementToggle>,
    manager: Res<PageManager>,
    mut query: Query<&mut Properties<TextProps>>,
) {
    if let Some(page) = manager.get("showcase") {
        let entity = page.get("checkbox-text");
        let mut text = query.get_mut(entity).unwrap();

        if toggle.matches_id("checkbox") {
            text.mutate(|props| {
                props.content = if toggle.state {
                    "Checked".to_string()
                } else {
                    "Not Checked".to_string()
                }
            });
        }
    }
}

fn switch(
    toggle: On<ElementToggle>,
    manager: Res<PageManager>,
    mut query: Query<&mut Properties<TextProps>>,
) {
    if let Some(page) = manager.get("showcase") {
        let entity = page.get("switch-text");
        let mut text = query.get_mut(entity).unwrap();

        if toggle.matches_id("switch") {
            text.mutate(|props| {
                props.content = if toggle.state {
                    "Toggled".to_string()
                } else {
                    "Not Toggled".to_string()
                }
            });
        }
    }
}

fn slider(set: On<ElementSet<f32>>, mut progress: Single<&mut Properties<ProgressBarProps>>) {
    if set.matches_id("slider") {
        progress.mutate(|props| props.value = set.value);
    }
}
