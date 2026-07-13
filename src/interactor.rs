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
    let root = TreeInteractorNode::create_mirroring_node(&tree.root, false);

    Self {
      tree,
      root,
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

  fn get_children_of_type(
    &self,
    ty: NodeDefinitionType,
    path: impl Iterator<Item = usize>,
  ) -> Option<impl Iterator<Item = &TreeNodeDefinition<T>> + '_> {
    let parent = self.select_node_via_path(path)?;
    Some(parent.children.iter().filter(move |kid| kid.ty == ty))
  }

  /// Pop out logic into extra function because it's complicated
  fn edit_picked(&mut self, ept: EditPickedType) -> Result<bool, TreeInteractionError> {
    let node = self.selected_node();
    // first, we need to handle special cases.
    // is this the AllDone node?
    if node.ty == NodeDefinitionType::AllDone {
      // get out of here!
      return Ok(true);
    } else if node.ty == NodeDefinitionType::Text {
      return Err(TreeInteractionError::TriedToEditUnpickableNode);
    }

    // clone so we can borrow later
    let parent_path = self.cursor_path[0..self.cursor_path.len() - 1].to_owned();
    let ideal_next_state = match ept {
      EditPickedType::Select => true,
      EditPickedType::Deselect => false,
      EditPickedType::Toggle => {
        let interactor = self.selected_interactor_node();
        let state = interactor.picked.expect("already handled unpickable nodes");
        !state
      }
    };

    // Handle `pick_children_needs_self`
    // handle parent.
    if ideal_next_state {
      let parent_node = self
        .select_node_via_path(parent_path.clone().into_iter())
        .expect("parent must be selectable");
      if parent_node.pick_children_needs_self {
        // we are picking this, so pick the parent
        let parent_inode = self
          .select_interactor_node_mut_via_path(parent_path.clone().into_iter())
          .unwrap();
        if let Some(ref mut slot) = parent_inode.picked {
          *slot = true;
        } else {
          panic!("do not set `pick_children_needs_self` on an unpickable node")
        }
      }
    }
    // handle children.
    // reborrow
    let node = self.selected_node();
    if node.pick_children_needs_self {
      if ideal_next_state {
        // if we pick this node, the first PickExactlyOne node must be picked.
        for idx in 0..node.children.len() {
          let new_path = {
            let mut path = self.cursor_path.clone();
            path.push(idx);
            path
          };
          let kid = self
            .select_node_via_path(new_path.clone().into_iter())
            .unwrap();
          if kid.ty == NodeDefinitionType::PickExactlyOne {
            let ikid_mut = self
              .select_interactor_node_mut_via_path(new_path.clone().into_iter())
              .unwrap();
            ikid_mut.picked = Some(true);
            break;
          }
        }
      } else {
        // if we unpick this node, all of its kids must be unpicked.
        for idx in 0..node.children.len() {
          let new_path = {
            let mut path = self.cursor_path.clone();
            path.push(idx);
            path
          };
          let ikid_mut = self
            .select_interactor_node_mut_via_path(new_path.clone().into_iter())
            .unwrap();
          if let Some(ref mut slot) = ikid_mut.picked {
            *slot = false;
          }
        }
      }
    }

    // reborrows
    let node = self.selected_node();
    let sib_count = node.children.len();
    match node.ty {
      NodeDefinitionType::Text | NodeDefinitionType::AllDone => unreachable!("already handled"),
      NodeDefinitionType::PickMany => {
        // Easy peasy
        let interactor_mut = self.selected_interactor_node_mut();
        interactor_mut.picked = Some(ideal_next_state);
        Ok(false)
      }
      NodeDefinitionType::PickUpToOne => {
        // make sure no siblings are picked
        if ideal_next_state {
          for i in 0..sib_count {
            let sib_path = parent_path.iter().cloned().chain(std::iter::once(i));
            let sib_node = self
              .select_node_via_path(sib_path.clone())
              .expect("if a node is selectable so is its parent");
            if sib_node.ty == NodeDefinitionType::PickUpToOne {
              let sib_inode_mut = self
                .select_interactor_node_mut_via_path(sib_path.clone())
                .unwrap();
              sib_inode_mut.picked = Some(false);
            }
          }
        }
        let interactor_mut = self.selected_interactor_node_mut();
        interactor_mut.picked = Some(ideal_next_state);
        Ok(false)
      }
      NodeDefinitionType::PickExactlyOne => {
        // ALWAYS make sure no siblings are picked.
        // If none of them were picked, then we can't unpick this one.
        let mut any_sib_picked = false;
        for i in 0..sib_count {
          let sib_path = parent_path.iter().cloned().chain(std::iter::once(i));
          let sib_node = self
            .select_node_via_path(sib_path.clone())
            .expect("if a node is selectable so is its parent");
          if sib_node.ty == NodeDefinitionType::PickExactlyOne {
            let sib_inode_mut = self
              .select_interactor_node_mut_via_path(sib_path.clone())
              .unwrap();
            if sib_inode_mut.picked == Some(true) {
              any_sib_picked = true;
            }
            sib_inode_mut.picked = Some(false);
          }
        }

        if !any_sib_picked && ideal_next_state == false {
          // do not set this node to unpicked; maintains invariants
          Err(TreeInteractionError::TriedToUnpickButNeedExactlyOne)
        } else {
          let interactor_mut = self.selected_interactor_node_mut();
          interactor_mut.picked = Some(ideal_next_state);
          Ok(false)
        }
      }
    }
  }
}

impl TreeInteractorNode {
  /// Create the default state of this node with enough `children` to match the shape of the node definition.
  fn create_mirroring_node<M>(node: &TreeNodeDefinition<M>, first_pick_exactly_one: bool) -> Self {
    let picked = match node.ty {
      NodeDefinitionType::PickMany => Some(false),
      NodeDefinitionType::PickUpToOne => Some(false),
      NodeDefinitionType::PickExactlyOne => {
        // This may be incorrect! What if the parent is also automatically not-picked
        // because *its* parent is PickExactlyOne?
        // hence, preproc step
        if first_pick_exactly_one {
          Some(true)
        } else {
          Some(false)
        }
      }
      NodeDefinitionType::Text | NodeDefinitionType::AllDone => None,
    };
    let mut children = Vec::new();
    let mut found_first_pick_exactly_one = false;
    // do NOT pick the first PickExactlyOne if this node is unpicked and has
    // `pick_children_need_self`
    let special_case_exclude_fpeo = picked == Some(false) && node.pick_children_needs_self;
    for idx in 0..node.children.len() {
      let kid = &node.children[idx];
      let found_fpeo = kid.ty == NodeDefinitionType::PickExactlyOne
        && !found_first_pick_exactly_one
        && !special_case_exclude_fpeo;
      if found_fpeo {
        found_first_pick_exactly_one = false;
      }
      children.push(TreeInteractorNode::create_mirroring_node(kid, found_fpeo));
    }
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
  /// [`TreeInteraction::EditPicked`] such that a family of [`NodeDefinitionType::PickExactlyOne`] nodes
  /// would have no picked siblings
  TriedToUnpickButNeedExactlyOne,
}
