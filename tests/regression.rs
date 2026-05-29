use treeversal::*;

use crate::helpers::assert_selected;

mod helpers;

#[test]
fn init_invariants() {
  // the first child of each PickOne node should be selected if mandatory
  let tree = helpers::example_tree();
  let interactor = TreeInteractor::new(tree);
  // do nothing ...
  assert_selected(&interactor, vec!["gamma.0", "delta.1.0"]);
}

#[test]
fn poke_around() {
  let tree = helpers::example_tree();
  let mut i = TreeInteractor::new(tree);

  i.interact(TreeInteraction::EnterNode).unwrap();
  i.interact(TreeInteraction::EditPicked(EditPickedType::Select))
    .unwrap();
  i.interact(TreeInteraction::SeekSibling { next: true })
    .unwrap();
  i.interact(TreeInteraction::EditPicked(EditPickedType::Select))
    .unwrap();
  assert_selected(&i, vec!["alpha.0", "alpha.1", "gamma.0", "delta.1.0"]);
  i.interact(TreeInteraction::SeekSibling { next: false })
    .unwrap();
  i.interact(TreeInteraction::EditPicked(EditPickedType::Deselect))
    .unwrap();
  assert_selected(&i, vec!["alpha.1", "gamma.0", "delta.1.0"]);
  i.interact(TreeInteraction::SeekSibling { next: false })
    .unwrap();
  i.interact(TreeInteraction::EditPicked(EditPickedType::Toggle))
    .unwrap();
  assert_selected(&i, vec!["alpha.1", "alpha.2", "gamma.0", "delta.1.0"]);

  i.interact(TreeInteraction::ExitNode).unwrap();

  // check picking PickMany cascading to children
  i.interact(TreeInteraction::EditPicked(EditPickedType::Select))
    .unwrap();
  let selected = i.get_all_selected_data();
  assert_eq!(
    selected,
    [&"alpha.0", &"alpha.1", &"alpha.2", &"gamma.0", &"delta.1.0"]
  );
  // selecting again should do nothing
  i.interact(TreeInteraction::EditPicked(EditPickedType::Select))
    .unwrap();
  assert_selected(
    &i,
    vec!["alpha.0", "alpha.1", "alpha.2", "gamma.0", "delta.1.0"],
  );
  i.interact(TreeInteraction::EnterNode).unwrap();
  i.interact(TreeInteraction::EditPicked(EditPickedType::Deselect))
    .unwrap();
  i.interact(TreeInteraction::ExitNode).unwrap();
  assert_selected(&i, vec!["alpha.1", "alpha.2", "gamma.0", "delta.1.0"]);
  i.interact(TreeInteraction::EditPicked(EditPickedType::Toggle))
    .unwrap();
  assert_selected(
    &i,
    vec!["alpha.0", "alpha.1", "alpha.2", "gamma.0", "delta.1.0"],
  );
  i.interact(TreeInteraction::EditPicked(EditPickedType::Toggle))
    .unwrap();
  assert_selected(&i, vec!["gamma.0", "delta.1.0"]);

  helpers::assert_invariants(&i);
}
