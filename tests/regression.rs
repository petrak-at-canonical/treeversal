use treeversal::*;

use std::assert_matches;

mod helpers;
use helpers::assert_selected;

#[test]
fn seeking_wraps() {
  use EditPickedType::*;
  use TreeInteraction::*;
  use treeversal::dsl::*;

  let tree = TreeDefinition::new(
    text("root")
      .with_child(pick_many("alpha"))
      .with_child(pick_many("beta"))
      .with_child(pick_many("gamma")),
  );
  let mut i = TreeInteractor::new(tree);
  assert_selected(&i, vec![]);

  i.interact(EditPicked(Select)).unwrap();
  assert_selected(&i, vec!["alpha"]);
  i.interact(SeekSibling { next: false }).unwrap();
  i.interact(EditPicked(Select)).unwrap();
  assert_selected(&i, vec!["alpha", "gamma"]);
  i.interact(SeekSibling { next: true }).unwrap();
  i.interact(EditPicked(Deselect)).unwrap();
  assert_selected(&i, vec!["gamma"]);
}

#[test]
fn edit_pick_type() {
  use EditPickedType::*;
  use TreeInteraction::*;
  use treeversal::dsl::*;

  let tree = TreeDefinition::new(text("root").with_child(pick_many("alpha")));
  let mut i = TreeInteractor::new(tree);
  assert_selected(&i, vec![]);

  // selection is idempotent
  i.interact(EditPicked(Select)).unwrap();
  assert_selected(&i, vec!["alpha"]);
  i.interact(EditPicked(Select)).unwrap();
  assert_selected(&i, vec!["alpha"]);
  // deselection is idempotent
  i.interact(EditPicked(Deselect)).unwrap();
  assert_selected(&i, vec![]);
  i.interact(EditPicked(Deselect)).unwrap();
  assert_selected(&i, vec![]);
  // toggle is not :]
  i.interact(EditPicked(Toggle)).unwrap();
  assert_selected(&i, vec!["alpha"]);
  i.interact(EditPicked(Toggle)).unwrap();
  assert_selected(&i, vec![]);
}

#[test]
fn pick_many_basic() {
  use EditPickedType::*;
  use TreeInteraction::*;
  use treeversal::dsl::*;

  let tree = TreeDefinition::new(
    text("root")
      .with_child(pick_many("alpha"))
      .with_child(pick_many("beta"))
      .with_child(pick_many("gamma")),
  );

  let mut i = TreeInteractor::new(tree);
  assert_selected(&i, vec![]);

  i.interact(EditPicked(Select)).unwrap();
  assert_selected(&i, vec!["alpha"]);
  i.interact(SeekSibling { next: true }).unwrap();
  i.interact(EditPicked(Select)).unwrap();
  assert_selected(&i, vec!["alpha", "beta"]);
  i.interact(SeekSibling { next: true }).unwrap();
  i.interact(EditPicked(Select)).unwrap();
  assert_selected(&i, vec!["alpha", "beta", "gamma"]);
  i.interact(SeekSibling { next: false }).unwrap();
  i.interact(EditPicked(Deselect)).unwrap();
  assert_selected(&i, vec!["alpha", "gamma"]);
}

#[test]
fn pick_many_text_cascade() {
  use EditPickedType::*;
  use TreeInteraction::*;
  use treeversal::dsl::*;

  let tree = TreeDefinition::new(
    text("root").with_child(
      text("stem")
        .with_child(pick_many("alpha"))
        .with_child(pick_many("beta"))
        .with_child(pick_many("gamma")),
    ),
  );

  let mut i = TreeInteractor::new(tree);
  assert_selected(&i, vec![]);

  i.interact(EditPicked(Select)).unwrap();
  assert_selected(&i, vec!["alpha", "beta", "gamma"]);
  // selecting is idempotent
  i.interact(EditPicked(Select)).unwrap();
  assert_selected(&i, vec!["alpha", "beta", "gamma"]);
  i.interact(EditPicked(Deselect)).unwrap();
  assert_selected(&i, vec![]);

  i.interact(EnterNode).unwrap();
  i.interact(SeekSibling { next: true }).unwrap();
  i.interact(EditPicked(Select)).unwrap();
  assert_selected(&i, vec!["beta"]);
  i.interact(ExitNode).unwrap();
  i.interact(EditPicked(Select)).unwrap();
  assert_selected(&i, vec!["alpha", "beta", "gamma"]);

  i.interact(EditPicked(Deselect)).unwrap();
  assert_selected(&i, vec![]);

  i.interact(EnterNode).unwrap();
  i.interact(SeekSibling { next: true }).unwrap();
  i.interact(EditPicked(Select)).unwrap();
  assert_selected(&i, vec!["beta"]);

  i.interact(ExitNode).unwrap();
  i.interact(EditPicked(Toggle)).unwrap();
  assert_selected(&i, vec!["alpha", "beta", "gamma"]);
  i.interact(EditPicked(Toggle)).unwrap();
  assert_selected(&i, vec![]);
}

#[test]
fn pick_up_to_one() {
  use EditPickedType::*;
  use TreeInteraction::*;
  use treeversal::dsl::*;

  let tree = TreeDefinition::new(
    text("root")
      .with_child(pick_up_to_one("alpha"))
      .with_child(pick_up_to_one("beta"))
      .with_child(pick_up_to_one("gamma")),
  );

  let mut i = TreeInteractor::new(tree);
  assert_selected(&i, vec![]);
  i.interact(SeekSibling { next: true }).unwrap();
  i.interact(EditPicked(Select)).unwrap();
  assert_selected(&i, vec!["beta"]);
  i.interact(EditPicked(Deselect)).unwrap();
  assert_selected(&i, vec![]);
  i.interact(SeekSibling { next: true }).unwrap();
  i.interact(EditPicked(Select)).unwrap();
  assert_selected(&i, vec!["gamma"]);
  i.interact(SeekSibling { next: true }).unwrap();
  i.interact(EditPicked(Select)).unwrap();
  assert_selected(&i, vec!["alpha"]);
}

#[test]
fn pick_exactly_one() {
  use EditPickedType::*;
  use TreeInteraction::*;
  use treeversal::dsl::*;

  let tree = TreeDefinition::new(
    text("root")
      .with_child(pick_exactly_one("alpha"))
      .with_child(pick_exactly_one("beta"))
      .with_child(pick_exactly_one("gamma")),
  );
  let mut i = TreeInteractor::new(tree);
  assert_selected(&i, vec!["alpha"]);
  i.interact(SeekSibling { next: true }).unwrap();
  i.interact(EditPicked(Select)).unwrap();
  assert_selected(&i, vec!["beta"]);
  let desel = i.interact(EditPicked(Deselect));
  assert_matches!(
    desel,
    Err(TreeInteractionError::TriedToUnpickButNeedExactlyOne)
  );
  assert_selected(&i, vec!["beta"]);
}

#[test]
fn error_editing_unpickable_text_node() {
  // A bare Text node with no PickMany children is not pickable.
  // Attempting EditPicked on it should return TriedToEditUnpickableNode.
  use EditPickedType::*;
  use TreeInteraction::*;
  use treeversal::dsl::*;

  let tree = TreeDefinition::new(
    text("root")
      .with_child(text("unpickable")),
  );
  let mut i = TreeInteractor::new(tree);
  let result = i.interact(EditPicked(Select));
  assert_matches!(
    result,
    Err(TreeInteractionError::TriedToEditUnpickableNode)
  );
  let result = i.interact(EditPicked(Toggle));
  assert_matches!(
    result,
    Err(TreeInteractionError::TriedToEditUnpickableNode)
  );
  let result = i.interact(EditPicked(Deselect));
  assert_matches!(
    result,
    Err(TreeInteractionError::TriedToEditUnpickableNode)
  );
}

#[test]
fn error_entering_node_with_no_children() {
  // Entering a leaf node (one with no children) should return NodeHasNoChildren.
  use TreeInteraction::*;
  use treeversal::dsl::*;

  let tree = TreeDefinition::new(
    text("root")
      .with_child(pick_many("leaf")),
  );
  let mut i = TreeInteractor::new(tree);
  let result = i.interact(EnterNode);
  assert_matches!(
    result,
    Err(TreeInteractionError::NodeHasNoChildren)
  );
}

#[test]
fn all_done_returns_true() {
  // Interacting with an AllDone node should return Ok(true),
  // signaling that the user is done and the caller should quit.
  use EditPickedType::*;
  use TreeInteraction::*;
  use treeversal::dsl::*;

  let tree = TreeDefinition::new(
    text("root")
      .with_child(pick_many("alpha"))
      .with_child(all_done("done")),
  );
  let mut i = TreeInteractor::new(tree);
  // cursor starts on "alpha"
  assert_eq!(i.interact(EditPicked(Select)), Ok(false));
  // move to "done"
  i.interact(SeekSibling { next: true }).unwrap();
  assert_eq!(i.interact(EditPicked(Select)), Ok(true));
  assert_eq!(i.interact(EditPicked(Toggle)), Ok(true));
  assert_eq!(i.interact(EditPicked(Deselect)), Ok(true));
}

#[test]
fn text_node_toggle_with_mixed_children() {
  // A Text node with both PickMany and non-PickMany children acts as a
  // checkbox for its PickMany children only. Toggle picks all PickMany
  // children if any are unpicked, and unpicks all if all are picked.
  // Non-PickMany children are unaffected by the Toggle itself, but picking
  // a PickUpToOne sibling will unpick PickMany siblings via sideways cascade.
  use EditPickedType::*;
  use TreeInteraction::*;
  use treeversal::dsl::*;

  let tree = TreeDefinition::new(
    text("root").with_child(
      text("stem")
        .with_child(pick_many("alpha"))
        .with_child(pick_up_to_one("beta"))
        .with_child(pick_many("gamma")),
    ),
  );
  let mut i = TreeInteractor::new(tree);
  assert_selected(&i, vec![]);

  // Toggle on: not all PickMany children are picked, so pick all PickMany
  i.interact(EditPicked(Toggle)).unwrap();
  assert_selected(&i, vec!["alpha", "gamma"]);

  // Enter stem, navigate to beta, and select it.
  // The PickUpToOne sideways cascade unpicks alpha and gamma.
  i.interact(EnterNode).unwrap();
  i.interact(SeekSibling { next: true }).unwrap();
  i.interact(EditPicked(Select)).unwrap();
  assert_selected(&i, vec!["beta"]);

  // Exit back to stem and Toggle: PickMany children are all unpicked,
  // so Toggle picks them all back. beta is unaffected by the Text toggle.
  i.interact(ExitNode).unwrap();
  i.interact(EditPicked(Toggle)).unwrap();
  assert_selected(&i, vec!["alpha", "beta", "gamma"]);

  // Toggle again: all PickMany are picked, so unpick them. beta stays.
  i.interact(EditPicked(Toggle)).unwrap();
  assert_selected(&i, vec!["beta"]);
}
