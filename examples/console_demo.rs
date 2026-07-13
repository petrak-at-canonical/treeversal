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
  use NodeDefinitionType::*;

  let bread_branch = TreeNodeDefinition::new_with_children(
    Text,
    msg("Pick a bread (mandatory)"),
    false,
    vec![
      TreeNodeDefinition::new(PickExactlyOne, msg("white"), false),
      TreeNodeDefinition::new(PickExactlyOne, msg("wheat"), true).with_child(
        TreeNodeDefinition::new(PickMany, msg("gluten-free wheat bread?"), false),
      ),
      TreeNodeDefinition::new(PickExactlyOne, msg("rye"), false),
    ],
  );

  let meat_branch = TreeNodeDefinition::new_with_children(
    Text,
    msg("Pick a meat"),
    false,
    vec![
      TreeNodeDefinition::new(PickUpToOne, msg("ham"), false),
      TreeNodeDefinition::new(PickUpToOne, msg("corned beef"), false),
      TreeNodeDefinition::new(PickUpToOne, msg("turkey"), false),
      TreeNodeDefinition::new(PickUpToOne, msg("chicken"), false),
    ],
  );

  let veggies_branch = TreeNodeDefinition::new(Text, msg("Pick vegetables"), false)
    .with_child(TreeNodeDefinition::new(PickMany, msg("lettuce"), false))
    .with_child(TreeNodeDefinition::new(PickMany, msg("tomatoes"), false))
    .with_child(TreeNodeDefinition::new(PickMany, msg("peppers"), false))
    .with_child(
      TreeNodeDefinition::new(PickMany, msg("onions"), true)
        .with_child(TreeNodeDefinition::new(
          PickExactlyOne,
          msg("red onions"),
          false,
        ))
        .with_child(TreeNodeDefinition::new(
          PickExactlyOne,
          msg("white onions"),
          false,
        ))
        .with_child(TreeNodeDefinition::new(
          PickExactlyOne,
          msg("grilled onions"),
          false,
        )),
    )
    .with_child(TreeNodeDefinition::new(PickMany, msg("avocado"), false));

  let sauce_branch = TreeNodeDefinition::new(Text, msg("Pick sauces"), false)
    .with_child(TreeNodeDefinition::new(PickMany, msg("mayonnaise"), false))
    .with_child(TreeNodeDefinition::new(
      PickMany,
      msg("barbeque sauce"),
      false,
    ))
    .with_child(TreeNodeDefinition::new(
      PickMany,
      msg("oil and vinegar"),
      false,
    ));

  let tree = TreeNodeDefinition::new(
    NodeDefinitionType::Text,
    msg("Customize your sandwich"),
    false,
  )
  .with_child(bread_branch)
  .with_child(meat_branch)
  .with_child(veggies_branch)
  .with_child(sauce_branch)
  .with_child(TreeNodeDefinition::new(
    NodeDefinitionType::AllDone,
    msg("Finished?"),
    false,
  ));
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
