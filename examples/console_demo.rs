use console::Key;
use treeversal::{
  NodeDefinitionType, TreeDefinition, TreeNodeDefinition,
  console_driver::{ConsoleDriver, Palette, StyledMsgAndData, TakeInput},
};

fn msg(s: impl AsRef<str>) -> StyledMsgAndData<()> {
  StyledMsgAndData {
    message: console::style(s.as_ref().to_owned()),
    data: (),
  }
}

pub fn main() {
  use treeversal::dsl::*;

  let bread_branch = TreeNodeDefinition::new_with_children(
    NodeDefinitionType::PickOneChild { mandatory: true },
    msg("Pick a bread (mandatory)"),
    vec![
      text(msg("white")),
      pick_many(msg("wheat")).with_child(text(msg("gluten-free wheat bread?"))),
      text(msg("rye")),
    ],
  );

  let meat_branch = TreeNodeDefinition::new_with_children(
    NodeDefinitionType::PickOneChild { mandatory: false },
    msg("Pick a meat"),
    vec![
      text(msg("ham")),
      text(msg("corned beef")),
      text(msg("turkey")),
      text(msg("chicken")),
    ],
  );

  // TODO: make onion type branch turn off if onions is deselected?
  let veggies_branch = pick_many(msg("Pick vegetables"))
    .with_child(text(msg("lettuce")))
    .with_child(text(msg("tomato")))
    .with_child(text(msg("peppers")))
    .with_child(
      pick_exactly_one(msg("onions"))
        .with_child(text(msg("red onions")))
        .with_child(text(msg("white onions")))
        .with_child(text(msg("grilled onions"))),
    )
    .with_child(text(msg("avocado")));

  let sauce_branch = pick_many(msg("Pick sauces"))
    .with_child(text(msg("mayonnaise")))
    .with_child(text(msg("barbeque sauce")))
    .with_child(text(msg("oil and vinegar")));

  let tree = TreeNodeDefinition::new(NodeDefinitionType::Text, msg("Customize your sandwich"))
    .with_child(bread_branch)
    .with_child(meat_branch)
    .with_child(veggies_branch)
    .with_child(sauce_branch)
    .with_child(all_done(msg("Finished?")));
  let tree = TreeDefinition::new(tree);

  let mut driver = ConsoleDriver::new_stdout(Palette::fancy(), tree);
  driver.print_tree();

  while let Ok(key) = driver.term.read_key() {
    if key == Key::CtrlC {
      return;
    }
    let finished = driver.take_input(key);
    if let Ok(TakeInput::Quit) = finished {
      break;
    }

    driver.print_tree();
  }

  let selected_names = driver
    .interactor
    .get_all_selected_data()
    .into_iter()
    .map(|smad| &smad.message)
    .collect::<Vec<_>>();
  println!("You selected the following nodes: {:?}", selected_names);
  println!(
    "(They were from these paths: {:?})",
    driver.interactor.get_all_selected_paths()
  );
}
