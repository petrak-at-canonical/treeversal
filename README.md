# tree-installer-sketch

Just a work in progress for now.

A simple CLI-based tool that lets you traverse and pick nodes in a tree, inspired by .MSI installer wizards.

## Unimplemented Features

- Figure out how to return the information to the programmer in a useful way
  - List of selected `TreeNodeDefinition.data`s?
  - List of selected paths?
- Add UX for finishing
  - Mandatory last child of the root makes it quit?
- Make `console_driver` rely on a feature flag

## Nice to Have Features

- Different themes for `console_driver` so you can print with box-drawing chars
  or whatever if you want.
- Ratatui driver
- Less ugly DSL
  - this may be impossible because this is rust not lisp
