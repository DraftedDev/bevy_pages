use crate::events::ElementSpawn;
use crate::props::Properties;
use crate::widgets::Widget;
use bevy::asset::AssetServer;
use bevy::color::Color;
use bevy::ecs::relationship::RelatedSpawnerCommands;
use bevy::prelude::{
    BorderColor, ChildOf, Component, Entity, GlobalTransform, Interaction, Transform,
};
use bevy::ui::{BackgroundColor, Node};
use rustc_hash::FxHashMap;
use smol_str::SmolStr;

/// A UI element representing a node in the UI hierarchy.
#[derive(Debug)]
pub struct Element {
    /// The element widget.
    pub widget: Box<dyn Widget>,
    /// The core visual, layout, and widget properties.
    pub props: Properties<ElementProps>,
    /// The optional ID of this element.
    pub id: Option<ElementId>,
    /// The children of this element.
    pub children: Vec<Element>,
}

impl Element {
    pub(crate) fn spawn(
        &self,
        parent: &mut RelatedSpawnerCommands<ChildOf>,
        assets: &AssetServer,
        reg: &mut ElementRegistry,
    ) -> Entity {
        let props = &self.props.default;

        let mut entity_cmd = parent.spawn((
            self.props.clone(),
            props.node.clone(),
            Interaction::None,
            Transform::default(),
            GlobalTransform::default(),
            BackgroundColor(props.bg_color.unwrap_or(Color::NONE)),
            props.border_color.unwrap_or(BorderColor::DEFAULT),
        ));

        if let Some(id) = &self.id {
            entity_cmd.insert(id.clone());
        }

        let root_entity = entity_cmd.id();

        let target_entity = self.widget.spawn(&mut entity_cmd, assets);

        if target_entity == root_entity {
            entity_cmd.with_children(|p| {
                for child in &self.children {
                    child.spawn(p, assets, reg);
                }
            });
        } else {
            parent
                .commands_mut()
                .entity(target_entity)
                .with_children(|p| {
                    for child in &self.children {
                        child.spawn(p, assets, reg);
                    }
                });
        }

        if let Some(id) = self.id() {
            reg.register_element(id, root_entity);
        }

        parent.commands_mut().trigger(ElementSpawn {
            entity: root_entity,
            id: self.id(),
        });

        root_entity
    }

    /// Returns the [ElementId] of the element, if it has one
    #[inline(always)]
    pub fn id(&self) -> Option<ElementId> {
        self.id.clone()
    }
}

/// Pure visual and layout properties for any element.
#[derive(Clone, Debug, Default)]
pub struct ElementProps {
    /// The node layout settings behind this element.
    pub node: Node,
    /// The background color of this element.
    pub bg_color: Option<Color>,
    /// The border color of this element.
    pub border_color: Option<BorderColor>,
}

/// A unique identifier for an element.
#[derive(Clone, Debug, PartialEq, Eq, Hash, Default, Component)]
pub struct ElementId(SmolStr);

impl ElementId {
    /// Creates a new [ElementId].
    #[inline(always)]
    pub fn new(s: impl AsRef<str>) -> Self {
        Self(SmolStr::new(s))
    }

    /// Creates a new [ElementId] from a static string.
    #[inline(always)]
    pub fn new_static(s: &'static str) -> Self {
        Self(SmolStr::new_static(s))
    }
}

impl From<&'static str> for ElementId {
    #[inline(always)]
    fn from(s: &'static str) -> Self {
        Self::new_static(s)
    }
}

#[derive(Clone, Debug)]
pub(crate) struct ElementRegistry {
    ids: FxHashMap<ElementId, Entity>,
}

impl ElementRegistry {
    #[inline(always)]
    pub(crate) fn new(cap: usize) -> Self {
        Self {
            ids: FxHashMap::with_capacity_and_hasher(cap, Default::default()),
        }
    }

    #[inline(always)]
    pub(crate) fn register_element(&mut self, id: ElementId, entity: Entity) {
        self.ids.insert(id, entity);
    }

    #[inline(always)]
    pub(crate) fn get_element(&self, id: ElementId) -> Option<Entity> {
        self.ids.get(&id).cloned()
    }
}
