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
  /// The behavior of this node.
  pub ty: NodeDefinitionType,
  /// The children.
  pub children: Vec<TreeNodeDefinition<T>>,
  /// If this is `true`, this node must be picked before any of its children can be picked.
  /// (Picking a child automatically picks this one.)
  pub pick_children_needs_self: bool,
}

impl<T> TreeNodeDefinition<T> {
  /// Create a new node with no children.
  ///
  /// Intended for builder-style use: see [`Self::with_child`].
  pub fn new(ty: NodeDefinitionType, data: T, pick_children_needs_self: bool) -> Self {
    Self::new_with_children(ty, data, pick_children_needs_self, Vec::new())
  }

  /// Create a new node with the following children.
  pub fn new_with_children(
    ty: NodeDefinitionType,
    data: T,
    pick_children_needs_self: bool,
    children: Vec<TreeNodeDefinition<T>>,
  ) -> Self {
    Self {
      data,
      ty,
      children,
      pick_children_needs_self,
    }
  }

  /// Append `child` to this node's children and return it.
  /// Intended for builder-style use.
  pub fn with_child(mut self, child: TreeNodeDefinition<T>) -> Self {
    self.children.push(child);
    self
  }

  /// Set `pick_children_needs_self`.
  ///
  /// Intended for builder-style use.
  pub fn with_pick_children_needs_self(mut self, pick_children_needs_self: bool) -> Self {
    self.pick_children_needs_self = pick_children_needs_self;
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

/// Determines the behavior of this node.
///
/// To avoid confusion, it's probably best to only have nodes of one type in a group.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum NodeDefinitionType {
  /// Any number of `PickAnyChild` nodes in a group may be picked.
  ///
  /// Displays with a square button: `[ ] Self`.
  PickMany,
  /// Exactly one `PickUpToOne` node in a group may be picked.
  ///
  /// Displays with a "radio button": `( ) Self`
  PickUpToOne,
  /// If there are any `PickExactlyOne` nodes in a group, one MUST be picked.
  /// These nodes do not interact with `PickUpToOne` nodes, which is confusing --
  /// you probably shouldn't have these as siblings.
  ///
  /// [`TreeNodeDefinition::pick_children_needs_self`] takes priority over this node's rules,
  /// so if the parent has `pick_children_needs_self` and is unpicked, then
  /// the siblings of this node may be unpicked..
  ///
  /// Displays with a "radio button": `( ) Self`
  PickExactlyOne,
  /// Just displays text. Not pickable.
  ///
  /// As a special case, if this has any `PickMany` children,
  /// then it will display with a square button.
  /// Picking the button will (un)pick all of its `PickMany` children.
  /// This does not actually influence any picking state on this node,
  /// it's just convenience.
  Text,
  /// The user picks this to indicate they are done.
  /// As convention, only the last child of the root should be this type.
  ///
  /// Not technically pickable; interacting with this node is special-cased.
  AllDone,
}
