use bevy::DefaultPlugins;
use bevy::app::{App, PluginGroup, Startup};
use bevy::asset::{AssetPlugin, AssetServer};
use bevy::camera::{Camera, Camera2d};
use bevy::prelude::{Commands, On, Res, ResMut, Transform};
use bevy_pages::PagesPlugin;
use bevy_pages::events::ElementClick;
use bevy_pages::manager::PageManager;

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
        .add_observer(handle_click)
        .run();
}

fn setup(mut commands: Commands, assets: Res<AssetServer>, mut manager: ResMut<PageManager>) {
    commands.spawn((Camera2d::default(), Camera::default(), Transform::default()));
    let main = assets.load("multi_page/main.xml");
    let dialog = assets.load("multi_page/dialog.xml");

    manager.spawn("main", main);
    manager.set_active("main", true);

    manager.spawn("dialog", dialog);
    // spawned pages are deactivated by default
}

fn handle_click(event: On<ElementClick>, mut manager: ResMut<PageManager>) {
    if let Some(_) = manager.get("main")
        && event.matches_id("open")
    {
        manager.set_active("main", false);
        manager.set_active("dialog", true);
    } else if let Some(_) = manager.get("dialog")
        && event.matches_id("close")
    {
        manager.set_active("dialog", false);
        manager.set_active("main", true);
    }
}
