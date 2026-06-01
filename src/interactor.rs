use getset::{CopyGetters, Getters};

use crate::{NodeDefinitionType, TreeDefinition, TreeNodeDefinition};

/// The mutable interface to interacting with a tree.
///
/// This struct internally "mirrors" the structure of a [`TreeNodeDefinition`].
///
/// ## Tree Paths
///
/// Traversing the tree is done with "paths." A path is a list of indices.
/// Traverse a node by indexing its children.
///
/// In other words,
/// - `[]` is the root node
/// - `[0]` is the first child of the root
/// - `[1]` is the second child of the root
/// - `[0, 1]` is the second child of the first child of the root
///
/// You may want to look at the implementation of [`TreeInteractor::select_node_via_path`] for more help.
#[derive(Getters)]
pub struct TreeInteractor<T> {
  /// The tree that this acts on.
  ///
  /// Depending on your use-case, you may want to clone the tree definition into here,
  /// or you can move it here and recover it afterwards.
  #[getset(get = "pub")]
  tree: TreeDefinition<T>,

  #[getset(get = "pub")]
  root: TreeInteractorNode,
  /// Path the current cursor took through the tree.
  /// Each index is the index of the [`TreeNodeDefinition`] in its parent.
  cursor_path: Vec<usize>,
}

/// How to interact with the tree.
#[derive(Debug, Clone, Copy)]
pub enum TreeInteraction {
  /// Change the picked-ness state of this node.
  EditPicked(EditPickedType),
  /// Go to this node's next/previous sibling. Wrapping.
  SeekSibling { next: bool },
  /// Go to this node's first child.
  EnterNode,
  /// Go to this node's parent.
  ExitNode,
}

/// How to change the picked state of a node.
///
/// See [`TreeInteraction::EditPicked`].
#[derive(Debug, Clone, Copy, Eq, PartialEq)]
pub enum EditPickedType {
  /// Pick this node
  Select,
  /// Unpick this node
  Deselect,
  /// Toggle the picked state of this node
  Toggle,
}

/// The state of an interaction with the tree.
///
/// Mostly public for the benefit of drivers.
#[derive(Getters, CopyGetters)]
pub struct TreeInteractorNode {
  /// If the node is not pickable (like a Text node), this is None.
  #[getset(get_copy = "pub")]
  picked: Option<bool>,
  #[getset(get = "pub")]
  children: Vec<TreeInteractorNode>,
}

impl<T> TreeInteractor<T> {
  /// Create a new tree.
  ///
  /// Panics if you pass it an empty tree. Why would you want to interact with an empty tree?
  pub fn new(tree: TreeDefinition<T>) -> TreeInteractor<T> {
    if tree.root.children.is_empty() {
      panic!("cannot create a TreeInteractor for an empty tree");
    }
    let root = TreeInteractorNode::create_mirroring_tree(&tree);

    Self {
      tree,
      root: root,
      // by default select the first child of the root
      cursor_path: vec![0],
    }
  }

  /// Get the path the cursor has taken through the tree.
  ///
  /// Each index is the index of the node in its parent.
  pub fn cursor_path(&self) -> &[usize] {
    &self.cursor_path
  }

  /// Traverse a path through the tree to the node definition.
  pub fn select_node_via_path(
    &self,
    mut path: impl Iterator<Item = usize>,
  ) -> Option<&TreeNodeDefinition<T>> {
    path.try_fold(&self.tree.root, |node, idx| node.children.get(idx))
  }

  /// The currently selected node, where the cursor is.
  pub fn selected_node(&self) -> &TreeNodeDefinition<T> {
    self
      .select_node_via_path(self.cursor_path.iter().copied())
      .expect("internal cursor must always have a valid path")
  }

  /// The currently selected node interactor, where the cursor is.
  fn select_interactor_node_mut_via_path(
    &mut self,
    mut path: impl Iterator<Item = usize>,
  ) -> Option<&mut TreeInteractorNode> {
    path.try_fold(&mut self.root, |node, idx| node.children.get_mut(idx))
  }

  fn selected_interactor_node_mut(&mut self) -> &mut TreeInteractorNode {
    let path = self.cursor_path.clone();
    self
      .select_interactor_node_mut_via_path(path.into_iter())
      .expect("internal cursor must always have a valid path")
  }

  /// Traverse a path through the tree to the node state.
  pub fn select_interactor_node_via_path(
    &self,
    mut path: impl Iterator<Item = usize>,
  ) -> Option<&TreeInteractorNode> {
    path.try_fold(&self.root, |node, idx| node.children.get(idx))
  }

  /// Get the currently selected node interaction state
  pub fn selected_interactor_node(&self) -> &TreeInteractorNode {
    self
      .select_interactor_node_via_path(self.cursor_path.iter().copied())
      .expect("internal cursor must always have a valid path")
  }

  /// Handle user input from the tree.
  ///
  /// Returns:
  /// - `Ok(false)` if the interaction was handled properly.
  /// - `Ok(true)` if the interaction was handled properly, and the user selected an [`NodeDefinitionType::AllDone`] node. The caller should quit the interaction loop.
  /// - `Err` if there was an error
  pub fn interact(&mut self, interaction: TreeInteraction) -> Result<bool, TreeInteractionError> {
    match interaction {
      TreeInteraction::EditPicked(ept) => self.edit_picked(ept),
      TreeInteraction::SeekSibling { next } => {
        let (&index_in_parent, path_to_parent) = self
          .cursor_path
          .split_last()
          .expect("`cursor` must be nonempty");
        let parent = self
          .select_node_via_path(path_to_parent.iter().copied())
          .expect("if a path is valid, the path with last popped is too");
        // will never panic on mod by zero because we check that a parent has children
        // before adding it to the cursor
        let new_idx = if next {
          (index_in_parent + 1) % parent.children.len()
        } else {
          (index_in_parent + parent.children.len() - 1) % parent.children.len()
        };
        self.cursor_path.pop();
        self.cursor_path.push(new_idx);
        Ok(false)
      }
      TreeInteraction::EnterNode => {
        let node = self.selected_node();
        if node.children.is_empty() {
          Err(TreeInteractionError::NodeHasNoChildren)
        } else {
          // Select its first child
          self.cursor_path.push(0);
          Ok(false)
        }
      }
      TreeInteraction::ExitNode => {
        if self.cursor_path.len() <= 1 {
          Err(TreeInteractionError::NodeHasNoParent)
        } else {
          self.cursor_path.pop();
          Ok(false)
        }
      }
    }
  }

  /// Consume self and return the tree definition.
  pub fn consume(self) -> TreeDefinition<T> {
    self.tree
  }

  /// Get every valid path through the tree.
  pub fn get_all_paths(&self) -> Vec<Vec<usize>> {
    self.root.get_all_paths_to_children_filtered(|_kid| true)
  }

  /// Get the path to every selected node.
  pub fn get_all_selected_paths(&self) -> Vec<Vec<usize>> {
    self
      .root
      .get_all_paths_to_children_filtered(|kid| kid.picked == Some(true))
  }

  /// Get the `data` associated with each picked node, as a flat list.
  pub fn get_all_selected_data(&self) -> Vec<&T> {
    let asp = self.get_all_selected_paths();
    asp
      .into_iter()
      .map(|path| {
        let node = self
          .select_node_via_path(path.into_iter())
          .expect("get_all_selected_paths must return valid paths");
        &node.data
      })
      .collect()
  }

  /// Pop out logic into extra function because it's complicated
  fn edit_picked(&mut self, ept: EditPickedType) -> Result<bool, TreeInteractionError> {
    // borrow immutably so we can mutably do it later
    let parent_path = self.cursor_path[0..self.cursor_path.len() - 1].to_owned();
    let parent = self
      .select_node_via_path(parent_path.iter().copied())
      .expect("if a path is valid, the path with last popped is too");
    let idx_in_parent = *self.cursor_path.last().expect("cursor must be nonempty");

    let node = self.selected_node();
    let interactor = self.selected_interactor_node();

    // first, we need to handle special cases.
    // is this the AllDone node?
    if node.ty == NodeDefinitionType::AllDone {
      // get out of here!
      return Ok(true);
    }

    // if the parent is a text node and this is a PickMany node, "picking" this
    // edits its children.
    if parent.ty == NodeDefinitionType::Text && node.ty == NodeDefinitionType::PickManyChildren {
      let new_child_state = match ept {
        EditPickedType::Select => true,
        EditPickedType::Deselect => false,
        EditPickedType::Toggle => {
          // if all of them are picked, turn it off
          // otherwise turn it on
          let every_kid_picked = interactor
            .children
            .iter()
            .all(|ikid| ikid.picked == Some(true));
          !every_kid_picked
        }
      };

      let interactor_mut = self.selected_interactor_node_mut();
      for kid in interactor_mut.children.iter_mut() {
        if let Some(ref mut picked) = kid.picked {
          *picked = new_child_state;
        }
        // else uh, dunno how that happened
      }

      return Ok(false);
    }

    let Some(picked) = interactor.picked else {
      return Err(TreeInteractionError::TriedToEditUnpickableNode);
    };
    let new_val = match ept {
      EditPickedType::Select => true,
      EditPickedType::Deselect => false,
      EditPickedType::Toggle => !picked,
    };

    // if any amount of picked is OK, then we are done.
    match parent.ty {
      NodeDefinitionType::Text | NodeDefinitionType::AllDone => {
        // uh not sure how this happened
        // please do not make children of AllDone nodes
        Ok(false)
      }
      NodeDefinitionType::PickManyChildren => {
        let remut = self.selected_interactor_node_mut();
        remut.picked = Some(new_val);
        Ok(false)
      }
      NodeDefinitionType::PickOneChild { mandatory } => {
        let parent_interactor_mut = self
          .select_interactor_node_mut_via_path(parent_path.iter().copied())
          .expect("if a path is valid, the path with last popped is too");

        for (idx, sib) in parent_interactor_mut.children.iter_mut().enumerate() {
          let is_me = idx == idx_in_parent;

          if is_me {
            // if we are trying to unpick this, but it's picked and one is
            // mandatory to be picked, prevent it.
            let prevent_unpicking = mandatory && picked == true && new_val == false;
            if !prevent_unpicking {
              sib.picked = Some(new_val);
            }
          } else {
            // if another one is picked, then unpick it.
            // this simulates "moving" the selection to the current one.
            // as a bonus, if more than one in a group is selected somehow,
            // this will fix it.
            if new_val == true && sib.picked == Some(true) {
              sib.picked = Some(false);
            }
          }
        }

        Ok(false)
      }
    }
  }
}

impl TreeInteractorNode {
  fn create_mirroring_tree<M>(tree: &TreeDefinition<M>) -> Self {
    let children = (0..tree.root.children.len())
      .map(|index| Self::create_mirroring_node(&tree.root, index))
      .collect();
    // The root node is never pickable
    Self {
      picked: None,
      children,
    }
  }

  /// Create the default state of this node with enough `children` to match the shape of the node definition
  fn create_mirroring_node<M>(parent: &TreeNodeDefinition<M>, index: usize) -> Self {
    let node = parent
      .children
      .get(index)
      .expect("`index` must be in range");
    let picked = match (parent.ty, node.ty) {
      (NodeDefinitionType::Text, NodeDefinitionType::PickManyChildren) => {
        // this type of node displays a checkbox, but it's not actually pickable,
        // the checkbox is there to reflect the state of the children.
        None
      }
      (NodeDefinitionType::Text | NodeDefinitionType::AllDone, _) => None,
      (NodeDefinitionType::PickOneChild { mandatory }, _) => {
        // auto-select the first node if mandatory
        if mandatory && index == 0 {
          Some(true)
        } else {
          Some(false)
        }
      }
      (NodeDefinitionType::PickManyChildren, _) => Some(false),
    };
    let children = (0..node.children.len())
      .map(|subindex| Self::create_mirroring_node(node, subindex))
      .collect();
    Self { picked, children }
  }

  // reminds me of data structures and algorithms class :]
  fn get_all_paths_to_children_filtered<F>(&self, f: F) -> Vec<Vec<usize>>
  where
    F: Fn(&TreeInteractorNode) -> bool + Copy,
  {
    let mut paths = Vec::new();
    for (idx, kid) in self.children().iter().enumerate() {
      let ok = f(kid);
      if ok {
        paths.push(vec![idx]);
      }

      let subpaths = kid.get_all_paths_to_children_filtered(f);
      for subpath in subpaths {
        let mut extended = vec![idx];
        extended.extend(subpath);
        paths.push(extended);
      }
    }
    paths
  }
}

/// What went wrong when you did an invalid operation on the tree
#[derive(Debug, Clone, Eq, PartialEq)]
pub enum TreeInteractionError {
  /// [`TreeInteraction::EditPicked`] on an unpickable node
  TriedToEditUnpickableNode,
  /// [`TreeInteraction::EnterNode`] on node with no children
  NodeHasNoChildren,
  /// [`TreeInteraction::ExitNode`] on a child of the root node
  NodeHasNoParent,
}
