use bevy::DefaultPlugins;
use bevy::app::{App, Startup};
use bevy::asset::AssetServer;
use bevy::camera::{Camera, Camera2d};
use bevy::prelude::{Commands, On, Query, Res, ResMut, Transform};
use bevy_pages::PagesPlugin;
use bevy_pages::events::ElementClick;
use bevy_pages::page::Page;
use bevy_pages::props::Properties;
use bevy_pages::spawner::PageSpawner;
use bevy_pages::widgets::text::TextProps;

fn main() {
    App::new()
        .add_plugins((DefaultPlugins, PagesPlugin::default()))
        .add_systems(Startup, setup)
        .add_observer(on_click)
        .run();
}

fn setup(mut commands: Commands, assets: Res<AssetServer>, mut spawner: ResMut<PageSpawner>) {
    commands.spawn((Camera2d::default(), Camera::default(), Transform::default()));
    let handle = assets.load("counter.xml");

    spawner.spawn(handle);
}

fn on_click(
    click: On<ElementClick>,
    page: Res<Page>,
    mut text_query: Query<&mut Properties<TextProps>>,
) {
    let counter_entity = page.get("counter");
    let mut counter_text = text_query.get_mut(counter_entity).unwrap();
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
