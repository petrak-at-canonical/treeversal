//! Helper builder functions to make making trees less of a headache
use crate::{NodeDefinitionType, TreeNodeDefinition};

pub fn text<T>(data: T) -> TreeNodeDefinition<T> {
  TreeNodeDefinition::new(NodeDefinitionType::Text, data, false)
}
pub fn pick_many<T>(data: T) -> TreeNodeDefinition<T> {
  TreeNodeDefinition::new(NodeDefinitionType::PickMany, data, false)
}

pub fn pick_up_to_one<T>(data: T) -> TreeNodeDefinition<T> {
  TreeNodeDefinition::new(NodeDefinitionType::PickUpToOne, data, false)
}

pub fn pick_exactly_one<T>(data: T) -> TreeNodeDefinition<T> {
  TreeNodeDefinition::new(NodeDefinitionType::PickExactlyOne, data, false)
}

pub fn all_done<T>(data: T) -> TreeNodeDefinition<T> {
  TreeNodeDefinition::new(NodeDefinitionType::AllDone, data, false)
}
