//! ```text
//! Customize your sandwich
//! ├─Pick a bread (mandatory)
//! │  ├─(o) white
//! │  ├─( ) wheat
//! │  │  └─[ ] gluten-free wheat bread?
//! │  └─( ) rye
//! ├─Pick a meat
//! │  ├─( ) ham
//! │  ├─( ) corned beef
//! │  ├─( ) turkey
//! │  └─( ) chicken
//! ├─[ ] Pick vegetables
//! │  ├─[ ] lettuce
//! │  ├─[ ] tomato
//! │  ├─[ ] peppers
//! │  ├─[ ] onions
//! │  │  ├─(o) red onions
//! │  │  ├─( ) white onions
//! │  │  └─( ) grilled onions
//! │  └─[ ] avocado
//! ├>[ ] Pick sauces
//! │  ├─[ ] mayonnaise
//! │  ├─[ ] barbeque sauce
//! │  └─[ ] oil and vinegar
//! └─{?} Finished?
//! ```
//!
//! A library for traversal and manipulation of a tree.
//! Create a tree of nodes as a [`TreeNodeDefinition`],
//! feed it to a [`TreeInteractor`],
//! and manipulate the tree by applying [`TreeInteraction`]s.

#![forbid(unsafe_code)]
// ^^ brownie points

mod interactor;
pub use interactor::*;
pub mod dsl;

#[cfg(feature = "console")]
pub mod console_driver;

/// Wrapper around the root node to make it clear what is the root and what is a child.
///
/// Generic over `T`, which is additional data associated with each node.
/// This is intended to be used with `String` or a CLI's rich string type,
/// and you could store any data you like alongside it.
/// See [`TreeNodeDefinition::data`]
#[derive(Debug, Clone)]
pub struct TreeDefinition<T> {
  pub root: TreeNodeDefinition<T>,
}

impl<T> TreeDefinition<T> {
  pub fn new(root: TreeNodeDefinition<T>) -> Self {
    Self { root }
  }
}

/// A definition of a tree, or one of its subnodes.
/// This serves as the "blueprint" for the behavior of a [`TreeInteractor`].
#[derive(Debug, Clone)]
pub struct TreeNodeDefinition<T> {
  /// Extra data associated with this node.
  pub data: T,
  /// The behavior of this node's children.
  pub ty: NodeDefinitionType,
  /// The children.
  pub children: Vec<TreeNodeDefinition<T>>,
}

impl<T> TreeNodeDefinition<T> {
  /// Create a new node with no children.
  ///
  /// Intended for builder-style use: see [`Self::with_child`].
  pub fn new(ty: NodeDefinitionType, data: T) -> Self {
    Self::new_with_children(ty, data, Vec::new())
  }

  /// Create a new node with the following children.
  pub fn new_with_children(
    ty: NodeDefinitionType,
    data: T,
    children: Vec<TreeNodeDefinition<T>>,
  ) -> Self {
    Self { data, ty, children }
  }

  /// Append `child` to this node's children and return it.
  /// Intended for builder-style use.
  pub fn with_child(mut self, child: TreeNodeDefinition<T>) -> Self {
    self.children.push(child);
    self
  }

  pub fn total_len(&self) -> usize {
    self.children.len()
      + self
        .children
        .iter()
        .map(TreeNodeDefinition::total_len)
        .sum::<usize>()
  }
}

/// Determines the behavior of this node's children.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum NodeDefinitionType {
  /// Just displays text. This node is not pickable.
  Text,
  /// Pick up to one, or exactly one, child.
  /// Displays its children with "radio buttons" in front of their name: `( ) MyChild`.
  ///
  /// If `mandatory` is true, the first child will be auto-picked.
  PickOneChild { mandatory: bool },
  /// Pick any number of children.
  /// Displays its children with square buttons in front of their name: `[ ] MyChild`.
  ///
  /// If this node's parent is [`NodeDefinitionType::Text`] node, then this node will also print
  /// with a square button in front of it, and selecting this node will (de)select all of its children.
  PickManyChildren,
  /// The user selects this node to indicate that they are done editing the tree.
  /// This should probably be the last child of the root.
  AllDone,
}
