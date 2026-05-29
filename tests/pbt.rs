// property based testing

mod helpers;

use fastrand::Rng;
use treeversal::{EditPickedType, TreeInteraction, TreeInteractor};

fn random_interaction(rand: &mut Rng) -> TreeInteraction {
  let interacts = [
    TreeInteraction::EditPicked(EditPickedType::Select),
    TreeInteraction::EditPicked(EditPickedType::Deselect),
    TreeInteraction::EditPicked(EditPickedType::Toggle),
    TreeInteraction::SeekSibling { next: true },
    TreeInteraction::SeekSibling { next: false },
    TreeInteraction::EnterNode,
    TreeInteraction::ExitNode,
  ];
  rand.choice(interacts).unwrap()
}

// Tree interactions can return errors but shouldn't panic
#[test]
fn hammer_tree_with_random_interactions() {
  for _ in 0..100 {
    let tree = helpers::example_tree();
    let mut interactor = TreeInteractor::new(tree);

    let mut rng = Rng::with_seed(0o7604);
    for _ in 0..10000 {
      let interaction = random_interaction(&mut rng);
      let _ignore = interactor.interact(interaction);
    }

    helpers::assert_invariants(&interactor);
  }
}
