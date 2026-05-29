//! Helper builder functions to make making trees less of a headache
use crate::{NodeDefinitionType, TreeNodeDefinition};

pub fn text<T>(data: T) -> TreeNodeDefinition<T> {
  TreeNodeDefinition::new(NodeDefinitionType::Text, data)
}
pub fn pick_many<T>(data: T) -> TreeNodeDefinition<T> {
  TreeNodeDefinition::new(NodeDefinitionType::PickManyChildren, data)
}

pub fn pick_up_to_one<T>(data: T) -> TreeNodeDefinition<T> {
  TreeNodeDefinition::new(NodeDefinitionType::PickOneChild { mandatory: false }, data)
}

pub fn pick_exactly_one<T>(data: T) -> TreeNodeDefinition<T> {
  TreeNodeDefinition::new(NodeDefinitionType::PickOneChild { mandatory: true }, data)
}

pub fn all_done<T>(data: T) -> TreeNodeDefinition<T> {
  TreeNodeDefinition::new(NodeDefinitionType::AllDone, data)
}
