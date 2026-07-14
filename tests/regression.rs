use treeversal::*;

use crate::helpers::assert_selected;
use std::assert_matches;

mod helpers;

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
fn pick_children_needs_self() {
  use EditPickedType::*;
  use TreeInteraction::*;
  use treeversal::dsl::*;

  let tree = TreeDefinition::new(
    text("root").with_child(
      pick_many("stem")
        .with_pick_children_needs_self(true)
        .with_child(pick_many("alpha"))
        .with_child(pick_many("beta"))
        .with_child(pick_many("gamma")),
    ),
  );
  let mut i = TreeInteractor::new(tree);
  assert_selected(&i, vec![]);
  i.interact(EnterNode).unwrap();
  i.interact(SeekSibling { next: true }).unwrap();
  i.interact(EditPicked(Select)).unwrap();
  assert_selected(&i, vec!["stem", "beta"]);
  i.interact(EditPicked(Deselect)).unwrap();
  assert_selected(&i, vec!["stem"]);
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
