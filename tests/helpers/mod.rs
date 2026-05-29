use treeversal::{NodeDefinitionType, TreeDefinition, TreeInteractor};

/// Make an example tree with nicely-named data
pub fn example_tree() -> TreeDefinition<&'static str> {
  use treeversal::dsl::*;
  let root = text("root")
    .with_child(
      pick_many("alpha")
        .with_child(text("alpha.0"))
        .with_child(text("alpha.1"))
        .with_child(text("alpha.2")),
    )
    .with_child(
      pick_up_to_one("beta")
        .with_child(text("beta.0"))
        .with_child(text("beta.1"))
        .with_child(text("beta.2"))
        .with_child(text("beta.3")),
    )
    .with_child(
      pick_exactly_one("gamma")
        .with_child(text("gamma.0"))
        .with_child(text("gamma.1"))
        .with_child(text("gamma.2")),
    )
    .with_child(
      pick_many("delta")
        .with_child(
          pick_many("delta.0")
            .with_child(text("delta.0.1"))
            .with_child(text("delta.0.2")),
        )
        .with_child(
          pick_exactly_one("delta.1")
            .with_child(text("delta.1.0"))
            .with_child(text("delta.1.1")),
        )
        .with_child(
          pick_many("delta.2")
            .with_child(text("delta.2.0"))
            .with_child(text("delta.2.1"))
            .with_child(text("delta.2.2")),
        ),
    );

  TreeDefinition::new(root)
}

pub fn assert_invariants<T>(interactor: &TreeInteractor<T>) {
  check_invariants_path(interactor, Vec::new());
}

fn check_invariants_path<T>(i: &TreeInteractor<T>, path: Vec<usize>) {
  let node = i.select_node_via_path(path.iter().copied()).unwrap();
  let inter = i
    .select_interactor_node_via_path(path.iter().copied())
    .unwrap();

  if let NodeDefinitionType::PickOneChild { mandatory } = node.ty {
    let picked_kid_count = inter
      .children()
      .iter()
      .filter(|kid| kid.picked() == Some(true))
      .count();

    if mandatory && picked_kid_count != 1 {
      panic!(
        "at node path {:?} had a mandatory PickOneChild node but no picks",
        &path
      );
    }
    if picked_kid_count >= 2 {
      panic!(
        "at node path {:?} had a mandatory PickOneChild node but 2 or more nodes were picked",
        &path
      );
    }
  }

  for idx in 0..inter.children().len() {
    let mut path2 = path.clone();
    path2.push(idx);
    check_invariants_path(i, path2);
  }
}

// rustc complains about this for some reason.
// even with ignore dead_code
// but not example_tree ???????????????/
#[ignore = "dead_code"]
pub fn assert_selected(i: &TreeInteractor<&'static str>, expected: Vec<&'static str>) {
  assert_invariants(i);
  let selected = i.get_all_selected_data();
  let expected_ref2 = expected.iter().collect::<Vec<_>>();
  assert_eq!(selected, expected_ref2);
}
