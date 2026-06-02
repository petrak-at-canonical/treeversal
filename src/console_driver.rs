//! Example driver for the tree using the `console` crate

use std::io::Write;

use console::{Key, Style, StyledObject, Term};

use crate::{
  NodeDefinitionType, TreeDefinition, TreeInteractor,
  interactor::{EditPickedType, TreeInteraction, TreeInteractionError},
};

/// Drives interaction with a [`TreeDefinition`] and [`TreeInteractor`] via the terminal.
pub struct ConsoleDriver<T> {
  pub term: console::Term,
  pub interactor: TreeInteractor<StyledMsgAndData<T>>,
  pub printed_once: bool,
  pub palette: Palette,
}

/// Wrapper for some data associated with a tree node, plus a message to display with the node.
#[derive(Clone, Debug)]
pub struct StyledMsgAndData<T> {
  pub message: StyledObject<String>,
  pub data: T,
}

impl<T> StyledMsgAndData<T> {
  pub fn new(message: StyledObject<String>, data: T) -> Self {
    Self { message, data }
  }

  /// Create with an unstyled string
  pub fn unstyled(message: impl AsRef<str>, data: T) -> Self {
    Self::new(console::style(message.as_ref().to_owned()), data)
  }
}

impl<T> ConsoleDriver<T> {
  /// Create a new driver.
  pub fn new(term: Term, palette: Palette, tree: TreeDefinition<StyledMsgAndData<T>>) -> Self {
    Self {
      term,
      palette,
      interactor: TreeInteractor::new(tree),
      printed_once: false,
    }
  }

  /// Create a new console driver that prints unbuffered to stdout.
  pub fn new_stdout(palette: Palette, tree: TreeDefinition<StyledMsgAndData<T>>) -> Self {
    Self::new(Term::stdout(), palette, tree)
  }

  /// Handle a key input from the user.
  ///
  /// Returns how it handled the input, or an error from interacting with the tree wrong.
  pub fn take_input(&mut self, key: Key) -> Result<TakeInput, TreeInteractionError> {
    let action = match key {
      Key::Char(' ') | Key::Enter => TreeInteraction::EditPicked(EditPickedType::Toggle),
      // mostly for testing purposes
      Key::Char('y' | 'Y') => TreeInteraction::EditPicked(EditPickedType::Select),
      Key::Char('n' | 'N') => TreeInteraction::EditPicked(EditPickedType::Deselect),

      Key::Char('h') | Key::ArrowLeft => TreeInteraction::ExitNode,
      Key::Char('j') | Key::ArrowDown => TreeInteraction::SeekSibling { next: true },
      Key::Char('k') | Key::ArrowUp => TreeInteraction::SeekSibling { next: false },
      Key::Char('l') | Key::ArrowRight => TreeInteraction::EnterNode,
      _ => return Ok(TakeInput::Ignored),
    };

    let quit = self.interactor.interact(action)?;
    Ok(if quit {
      TakeInput::Quit
    } else {
      TakeInput::Accepted
    })
  }

  /// Print the whole tree to the terminal. This includes the user's cursor and selections.
  pub fn print_tree(&mut self) {
    if self.printed_once {
      self
        .term
        .clear_last_lines(self.interactor.tree().root.total_len() + 1)
        .unwrap();
    }
    self.print_node_by_path(Vec::new(), String::new());
    self.printed_once = true;
  }

  /// `scaffolding` is the leading pipes and spaces.
  ///
  /// Also directs the printing all of its children.
  fn print_node_by_path(&self, print_path: Vec<usize>, mut scaffolding: String) {
    let node = self
      .interactor
      .select_node_via_path(print_path.iter().copied())
      .unwrap();
    let interactor = self
      .interactor
      .select_interactor_node_via_path(print_path.iter().copied())
      .unwrap();

    let mb_parent = if print_path.is_empty() {
      None
    } else {
      Some(
        self
          .interactor
          .select_node_via_path(print_path[0..print_path.len() - 1].iter().copied())
          .unwrap(),
      )
    };

    let this_is_selected = &print_path == self.interactor.cursor_path();
    let additional_style = if this_is_selected {
      Style::new().bold()
    } else {
      Style::new()
    };

    // If this is not the root node, print leader characters
    // to indicate the tree structure
    if !print_path.is_empty() {
      let index_in_parent = print_path[print_path.len() - 1];
      let last_in_parent = index_in_parent == mb_parent.unwrap().children.len() - 1;

      let pipe = if last_in_parent {
        self.palette.pipe_corner
      } else {
        self.palette.pipe_branch
      };
      let indicator = if this_is_selected {
        &self.palette.selected_connector
      } else {
        &self.palette.unselected_connector
      };
      // TODO: does it look better with or without a space before the title
      let line = format!("{}{}{}", &scaffolding, pipe, indicator);
      write!(&self.term, "{}", additional_style.apply_to(line)).unwrap();
    }

    // Possibly write the checkbox
    let mb_parent_ty = mb_parent.map(|p| p.ty);
    match (mb_parent_ty, node.ty) {
      // Special case 1: all done
      (_, NodeDefinitionType::AllDone) => {
        write!(
          &self.term,
          "{}",
          additional_style.apply_to(&self.palette.all_done)
        )
        .unwrap();
      }
      // Special case 2: half-slash
      (None | Some(NodeDefinitionType::Text), NodeDefinitionType::PickManyChildren) => {
        let kid_picked_count = interactor
          .children()
          .iter()
          .filter(|ikid| ikid.picked() == Some(true))
          .count();
        let boxx = if kid_picked_count == 0 {
          &self.palette.unpicked_manybox
        } else if kid_picked_count == interactor.children().len() {
          &self.palette.picked_manybox
        } else {
          &self.palette.some_picked_manybox
        };
        write!(&self.term, "{}", additional_style.apply_to(boxx)).unwrap();
      }
      (Some(NodeDefinitionType::PickManyChildren), _) => {
        let boxx = match interactor.picked() {
          Some(true) => &self.palette.picked_manybox,
          Some(false) => &self.palette.unpicked_manybox,
          // oh no
          None => &self.palette.error_box,
        };
        write!(&self.term, "{}", additional_style.apply_to(boxx)).unwrap();
      }
      (Some(NodeDefinitionType::PickOneChild { .. }), _) => {
        let boxx = match interactor.picked() {
          Some(true) => &self.palette.picked_onebox,
          Some(false) => &self.palette.unpicked_onebox,
          // oh no
          None => &self.palette.error_box,
        };
        write!(&self.term, "{}", additional_style.apply_to(boxx)).unwrap();
      }
      // draw nothing
      (None | Some(NodeDefinitionType::Text) | Some(NodeDefinitionType::AllDone), _) => {}
    }

    // print the node!
    writeln!(
      &self.term,
      "{}",
      additional_style.apply_to(&node.data.message)
    )
    .unwrap();

    // consider extending the scaffolding
    if let Some(parent) = mb_parent {
      let index_in_parent = print_path[print_path.len() - 1];
      let last_in_parent = parent.children.len() == index_in_parent;
      let scaffold_char = if last_in_parent {
        ' '
      } else {
        self.palette.pipe_vert
      };
      scaffolding += &format!("{}  ", scaffold_char);
    }

    for (idx, _) in node.children.iter().enumerate() {
      let mut path2 = print_path.clone();
      path2.push(idx);
      self.print_node_by_path(path2, scaffolding.clone());
    }
  }
}

/// What happened after the user presses a key.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TakeInput {
  /// The key was handled and there was no error.
  Accepted,
  /// The key does not control the tree
  Ignored,
  /// The key was handled, and a [`NodeDefinitionType::AllDone`] was touched
  Quit,
}

/// How to draw each part of the tree
#[derive(Debug, Clone)]
pub struct Palette {
  /// Display an unchecked checkbox
  pub unpicked_manybox: String,
  /// Display a half-selected checkbox
  pub some_picked_manybox: String,
  /// Display a checked checkbox
  pub picked_manybox: String,
  /// Display an upicked radio button
  pub unpicked_onebox: String,
  /// Display a picked radio button
  pub picked_onebox: String,
  /// Used to indicate an invalid selection state as a debug tool.
  /// If you see this, something has gone wrong
  pub error_box: String,

  /// Char used to extend the tree vertically
  pub pipe_vert: char,
  /// Char to indicate that the tree branches
  pub pipe_branch: char,
  /// Char to indicate last branch of a node
  pub pipe_corner: char,
  /// Connector between the pipes and the node
  pub unselected_connector: String,
  /// Connector between the pipes and the node when the cursor is there
  pub selected_connector: String,

  /// Leading "box" for the finishing node
  pub all_done: String,
}

impl Default for Palette {
  fn default() -> Self {
    Self {
      unpicked_manybox: "[ ] ".to_string(),
      some_picked_manybox: "[/] ".to_string(),
      picked_manybox: "[X] ".to_string(),
      unpicked_onebox: "( ) ".to_string(),
      picked_onebox: "(o) ".to_string(),
      error_box: "[!] ".to_string(),
      pipe_vert: '|',
      pipe_branch: '|',
      pipe_corner: '`',
      unselected_connector: "-".to_string(),
      selected_connector: ">".to_string(),
      all_done: "{?} ".to_string(),
    }
  }
}

impl Palette {
  /// Draw the tree with fancy Unicode characters
  pub fn fancy() -> Self {
    Self {
      unpicked_manybox: "[ ] ".to_string(),
      some_picked_manybox: "[/] ".to_string(),
      picked_manybox: "[X] ".to_string(),
      unpicked_onebox: "( ) ".to_string(),
      picked_onebox: "(o) ".to_string(),
      error_box: "[!] ".to_string(),
      pipe_vert: '│',
      pipe_branch: '├',
      pipe_corner: '└',
      unselected_connector: "─".to_string(),
      selected_connector: ">".to_string(),
      all_done: "{?} ".to_string(),
    }
  }
}
