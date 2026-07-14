use treeversal::{TreeDefinition, TreeInteractor};

// rustc ignores my ignores for some reason

/// Make an example tree with nicely-named data
#[ignore = "dead_code"]
pub fn example_tree() -> TreeDefinition<&'static str> {
  use treeversal::dsl::*;
  let root = text("root")
    .with_child(
      text("alpha")
        .with_child(pick_many("alpha.0"))
        .with_child(pick_many("alpha.1"))
        .with_child(pick_many("alpha.2")),
    )
    .with_child(
      text("beta")
        .with_child(pick_up_to_one("beta.0"))
        .with_child(pick_up_to_one("beta.1"))
        .with_child(pick_up_to_one("beta.2"))
        .with_child(pick_up_to_one("beta.3")),
    )
    .with_child(
      text("gamma")
        .with_child(pick_exactly_one("gamma.0"))
        .with_child(pick_exactly_one("gamma.1"))
        .with_child(pick_exactly_one("gamma.2")),
    )
    .with_child(
      text("delta")
        .with_child(
          text("delta.0")
            .with_child(pick_many("delta.0.1"))
            .with_child(pick_many("delta.0.2")),
        )
        .with_child(
          text("delta.1")
            .with_child(pick_exactly_one("delta.1.0"))
            .with_child(pick_exactly_one("delta.1.1")),
        )
        .with_child(
          text("delta.2")
            .with_child(pick_many("delta.2.0"))
            .with_child(pick_many("delta.2.1"))
            .with_child(pick_many("delta.2.2")),
        ),
    );

  TreeDefinition::new(root)
}

#[ignore = "dead_code"]
pub fn assert_selected(i: &TreeInteractor<&'static str>, expected: Vec<&'static str>) {
  let selected = i.get_all_selected_data();
  let expected_ref2 = expected.iter().collect::<Vec<_>>();
  assert_eq!(selected, expected_ref2);
}
