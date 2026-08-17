use bevy::DefaultPlugins;
use bevy::app::{App, Startup};
use bevy::asset::AssetServer;
use bevy::camera::{Camera, Camera2d};
use bevy::prelude::{Commands, On, Query, Res, ResMut, Single, Transform};
use bevy_pages::PagesPlugin;
use bevy_pages::events::{ElementClick, ElementSet, ElementToggle};
use bevy_pages::page::Page;
use bevy_pages::props::Properties;
use bevy_pages::spawner::PageSpawner;
use bevy_pages::widgets::progress_bar::ProgressBarProps;
use bevy_pages::widgets::text::TextProps;

fn main() {
    App::new()
        .add_plugins((DefaultPlugins, PagesPlugin::default()))
        .add_systems(Startup, setup)
        .add_observer(counter)
        .add_observer(checkbox)
        .add_observer(slider)
        .run();
}

fn setup(mut commands: Commands, assets: Res<AssetServer>, mut spawner: ResMut<PageSpawner>) {
    commands.spawn((Camera2d::default(), Camera::default(), Transform::default()));
    let handle = assets.load("showcase.xml");

    spawner.spawn(handle);
}

fn counter(click: On<ElementClick>, page: Res<Page>, mut query: Query<&mut Properties<TextProps>>) {
    let counter_entity = page.get("counter");
    let mut props = query.get_mut(counter_entity).unwrap();
    let counter = props.default.content.parse::<i32>().unwrap();

    if click.matches_id("increment") {
        props.mutate(|props| props.content = (counter + 1).to_string());
    }

    if click.matches_id("decrement") {
        props.mutate(|props| props.content = (counter - 1).to_string());
    }
}

fn checkbox(
    toggle: On<ElementToggle>,
    page: Res<Page>,
    mut query: Query<&mut Properties<TextProps>>,
) {
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

fn slider(set: On<ElementSet<f32>>, mut progress: Single<&mut Properties<ProgressBarProps>>) {
    if set.matches_id("slider") {
        progress.mutate(|props| props.value = set.value);
    }
}
