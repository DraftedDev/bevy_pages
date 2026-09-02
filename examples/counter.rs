use bevy::DefaultPlugins;
use bevy::app::{App, PluginGroup, Startup};
use bevy::asset::{AssetPlugin, AssetServer};
use bevy::camera::{Camera, Camera2d};
use bevy::prelude::{Commands, On, Query, Res, ResMut, Transform};
use bevy_pages::PagesPlugin;
use bevy_pages::events::ElementClick;
use bevy_pages::manager::PageManager;
use bevy_pages::props::Properties;
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
        .add_observer(on_click)
        .run();
}

fn setup(mut commands: Commands, assets: Res<AssetServer>, mut manager: ResMut<PageManager>) {
    commands.spawn((Camera2d::default(), Camera::default(), Transform::default()));
    let handle = assets.load("counter.xml");

    manager.spawn("counter", handle);
    manager.set_active("counter", true);
}

fn on_click(
    click: On<ElementClick>,
    manager: Res<PageManager>,
    mut query: Query<&mut Properties<TextProps>>,
) {
    if let Some(page) = manager.get("counter") {
        let counter_entity = page.get("counter");
        let mut counter_text = query.get_mut(counter_entity).unwrap();
        let counter = counter_text.default.content.parse::<i32>().unwrap();

        if click.matches_id("increment") {
            counter_text.mutate(|props| props.content = (counter + 1).to_string());
        }

        if click.matches_id("decrement") {
            counter_text.mutate(|props| props.content = (counter - 1).to_string());
        }

        if click.matches_id("reset") {
            counter_text.mutate(|props| props.content = "0".to_string());
        }
    }
}
