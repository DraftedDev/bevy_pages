use crate::element::{ElementActive, ElementId, ElementProps, ElementState};
use crate::events::{ElementClick, ElementHover};
use crate::props::Properties;
use bevy::color::Color;
use bevy::prelude::*;
use bevy::ui::{BackgroundColor, BorderColor, Node};

/// Tracks the interaction state from the previous frame.
#[derive(Component, Debug, Copy, Clone, PartialEq, Eq, Deref, DerefMut)]
pub struct PreviousInteraction(pub Interaction);

/// System set grouping UI layout, input interaction, and scrolling systems.
#[derive(Copy, Clone, Debug, PartialEq, Eq, Hash, SystemSet)]
pub struct PageSystemSet;

pub(crate) fn update_state(
    mut commands: Commands,
    query: Query<(Entity, &ElementState), Changed<ElementState>>,
    children_query: Query<&Children, With<ElementState>>,
) {
    fn update_children(
        parent: Entity,
        is_active: bool,
        commands: &mut Commands,
        children_query: &Query<&Children, With<ElementState>>,
    ) {
        let Ok(children) = children_query.get(parent) else {
            return;
        };

        for child in children.iter() {
            if is_active {
                commands
                    .entity(child)
                    .insert((ElementActive, ElementState::Active));
            } else {
                commands
                    .entity(child)
                    .remove::<ElementActive>()
                    .insert(ElementState::Inactive);
            }

            update_children(child, is_active, commands, children_query);
        }
    }

    for (entity, state) in &query {
        let is_active = *state == ElementState::Active;

        if is_active {
            commands.entity(entity).insert(ElementActive);
        } else {
            commands.entity(entity).remove::<ElementActive>();
        }

        update_children(entity, is_active, &mut commands, &children_query);
    }
}

pub(crate) fn interactions(
    mut commands: Commands,
    mut query: Query<
        (
            Entity,
            &Interaction,
            Option<&PreviousInteraction>,
            &Properties<ElementProps>,
            Option<&ElementId>,
            &mut Node,
            &mut BackgroundColor,
            &mut BorderColor,
        ),
        (
            With<ElementActive>,
            Or<(Changed<Interaction>, Added<ElementActive>)>,
        ),
    >,
) {
    for (e, i, prev_i, props, id, mut node, mut bg_color, mut border_color) in &mut query {
        let previous = prev_i.map(|p| p.0).unwrap_or(Interaction::None);

        if previous == Interaction::Pressed && *i != Interaction::Pressed {
            commands.trigger(ElementClick {
                entity: e,
                id: id.cloned(),
            });
        }

        let props = match i {
            Interaction::Pressed => &props.click,
            Interaction::Hovered => {
                if previous != Interaction::Hovered {
                    commands.trigger(ElementHover {
                        entity: e,
                        id: id.cloned(),
                    });
                }
                &props.hover
            }
            Interaction::None => &props.default,
        };

        let target_bg = props.bg_color.unwrap_or(Color::NONE);
        let target_border = props.border_color.unwrap_or_default();

        crate::set_if_changed!(
            bg_color.0, target_bg;
            *border_color, target_border;
            *node, props.node => props.node.clone();
        );

        commands.entity(e).insert(PreviousInteraction(*i));
    }
}

#[cfg(feature = "hot-reload")]
pub(crate) fn hot_reload(
    mut events: MessageReader<AssetEvent<crate::page::Page>>,
    mut manager: ResMut<crate::manager::PageManager>,
    assets: Res<AssetServer>,
) {
    for event in events.read() {
        let id = match event {
            AssetEvent::Modified { id } => Some(id),
            AssetEvent::LoadedWithDependencies { id } => Some(id),
            _ => None,
        };

        if let Some(id) = id
            && let Some(handle) = assets.get_id_handle(*id)
        {
            manager.reload(handle);
        }
    }
}
