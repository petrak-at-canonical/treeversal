use treeversal::*;

mod helpers;
use helpers::assert_selected;

#[test]
fn basic_test() {
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
fn unpicking_parent() {
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
  i.interact(ExitNode).unwrap();
  i.interact(EditPicked(Deselect)).unwrap();
  assert_selected(&i, vec![]);
}

#[test]
fn multiple_children() {
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
  i.interact(EditPicked(Select)).unwrap();
  assert_selected(&i, vec!["stem", "alpha"]);
  i.interact(SeekSibling { next: true }).unwrap();
  i.interact(EditPicked(Select)).unwrap();
  assert_selected(&i, vec!["stem", "alpha", "beta"]);
  i.interact(SeekSibling { next: true }).unwrap();
  i.interact(EditPicked(Select)).unwrap();
  assert_selected(&i, vec!["stem", "alpha", "beta", "gamma"]);
  i.interact(ExitNode).unwrap();
  i.interact(EditPicked(Deselect)).unwrap();
  assert_selected(&i, vec![]);
}

#[test]
fn pick_exactly_one_children_auto_picked_when_parent_picked() {
  // When a pick_children_needs_self node is picked and has PickExactlyOne
  // children, the first PickExactlyOne child is auto-picked to satisfy the
  // "exactly one must be picked" invariant.
  use EditPickedType::*;
  use TreeInteraction::*;
  use treeversal::dsl::*;

  let tree = TreeDefinition::new(
    text("root").with_child(
      pick_many("stem")
        .with_pick_children_needs_self(true)
        .with_child(pick_exactly_one("alpha"))
        .with_child(pick_exactly_one("beta"))
        .with_child(pick_exactly_one("gamma")),
    ),
  );
  let mut i = TreeInteractor::new(tree);
  // Initially unpicked: pick_children_needs_self prevents auto-picking
  // the first PickExactlyOne child at initialization.
  assert_selected(&i, vec![]);

  // Pick the parent directly; the first PickExactlyOne child auto-picks.
  i.interact(EditPicked(Select)).unwrap();
  assert_selected(&i, vec!["stem", "alpha"]);

  // Switch the PickExactlyOne selection to another sibling.
  i.interact(EnterNode).unwrap();
  i.interact(SeekSibling { next: true }).unwrap();
  i.interact(EditPicked(Select)).unwrap();
  assert_selected(&i, vec!["stem", "beta"]);

  // Unpick the parent; all children should be unpicked.
  i.interact(ExitNode).unwrap();
  i.interact(EditPicked(Deselect)).unwrap();
  assert_selected(&i, vec![]);
}

#[test]
fn pick_exactly_one_children_not_auto_picked_at_init() {
  // At initialization, a PickExactlyOne group normally has the first child
  // picked. But if the parent has pick_children_needs_self and is unpicked,
  // the first PickExactlyOne child should NOT be auto-picked, since the
  // parent being unpicked overrides the "exactly one" invariant.
  use treeversal::dsl::*;

  let tree = TreeDefinition::new(
    text("root").with_child(
      pick_many("stem")
        .with_pick_children_needs_self(true)
        .with_child(pick_exactly_one("alpha"))
        .with_child(pick_exactly_one("beta")),
    ),
  );
  let i = TreeInteractor::new(tree);
  // None picked: the "exactly one" invariant is suspended because
  // the parent is unpicked with pick_children_needs_self.
  assert_selected(&i, vec![]);
}

#[test]
fn nested_pick_children_needs_self() {
  // When both a parent and grandparent have pick_children_needs_self,
  // picking a grandchild should auto-pick both the parent and the
  // grandparent via the recursive cascade_invariants_up.
  use EditPickedType::*;
  use TreeInteraction::*;
  use treeversal::dsl::*;

  let tree = TreeDefinition::new(
    text("root").with_child(
      pick_many("grandparent")
        .with_pick_children_needs_self(true)
        .with_child(
          pick_many("parent")
            .with_pick_children_needs_self(true)
            .with_child(pick_many("child")),
        ),
    ),
  );
  let mut i = TreeInteractor::new(tree);
  assert_selected(&i, vec![]);

  // Enter grandparent, then parent, then pick child.
  // All three should become picked.
  i.interact(EnterNode).unwrap();
  i.interact(EnterNode).unwrap();
  i.interact(EditPicked(Select)).unwrap();
  assert_selected(&i, vec!["grandparent", "parent", "child"]);

  // Unpick the grandparent; all descendants should be unpicked.
  i.interact(ExitNode).unwrap();
  i.interact(ExitNode).unwrap();
  i.interact(EditPicked(Deselect)).unwrap();
  assert_selected(&i, vec![]);
}

#[test]
fn pick_up_to_one_sideways_cascade_keeps_parent_picked() {
  // When a pick_children_needs_self parent has PickUpToOne children,
  // picking one child picks the parent, then picking a sibling unpicks
  // the first via sideways cascade but the parent stays picked since
  // a child is still selected.
  use EditPickedType::*;
  use TreeInteraction::*;
  use treeversal::dsl::*;

  let tree = TreeDefinition::new(
    text("root").with_child(
      pick_many("stem")
        .with_pick_children_needs_self(true)
        .with_child(pick_up_to_one("alpha"))
        .with_child(pick_up_to_one("beta"))
        .with_child(pick_up_to_one("gamma")),
    ),
  );
  let mut i = TreeInteractor::new(tree);
  assert_selected(&i, vec![]);

  // Pick alpha: enter stem first, then select child (parent auto-picked)
  i.interact(EnterNode).unwrap();
  i.interact(EditPicked(Select)).unwrap();
  assert_selected(&i, vec!["stem", "alpha"]);

  // Pick beta: alpha unpicked via sideways cascade, parent stays picked
  i.interact(SeekSibling { next: true }).unwrap();
  i.interact(EditPicked(Select)).unwrap();
  assert_selected(&i, vec!["stem", "beta"]);

  // Deselect beta: parent stays picked (as with any pick_children_needs_self)
  i.interact(EditPicked(Deselect)).unwrap();
  assert_selected(&i, vec!["stem"]);

  // Exit stem to test parent unpick
  i.interact(ExitNode).unwrap();
  i.interact(EditPicked(Deselect)).unwrap();
  assert_selected(&i, vec![]);
}

#[test]
fn toggle_child_under_pick_children_needs_self() {
  // Toggling a child under a pick_children_needs_self parent:
  // - Toggle on picks the child and auto-picks the parent
  // - Toggle off unpicks the child but the parent stays picked
  // Toggling the parent itself:
  // - Toggle off unpicks the parent and cascades to unpick all children
  use EditPickedType::*;
  use TreeInteraction::*;
  use treeversal::dsl::*;

  let tree = TreeDefinition::new(
    text("root").with_child(
      pick_many("stem")
        .with_pick_children_needs_self(true)
        .with_child(pick_many("alpha"))
        .with_child(pick_many("beta")),
    ),
  );
  let mut i = TreeInteractor::new(tree);
  assert_selected(&i, vec![]);

  // Toggle child on: enter stem first, then toggle child (picks child and parent)
  i.interact(EnterNode).unwrap();
  i.interact(EditPicked(Toggle)).unwrap();
  assert_selected(&i, vec!["stem", "alpha"]);

  // Toggle child off: unpicks child, parent stays
  i.interact(EditPicked(Toggle)).unwrap();
  assert_selected(&i, vec!["stem"]);

  // Pick a child again so we have something to cascade
  i.interact(SeekSibling { next: true }).unwrap();
  i.interact(EditPicked(Select)).unwrap();
  assert_selected(&i, vec!["stem", "beta"]);

  // Toggle parent off: cascades, unpicks all children
  i.interact(ExitNode).unwrap();
  i.interact(EditPicked(Toggle)).unwrap();
  assert_selected(&i, vec![]);
}
