# treeversal

[![Crates.io Version](https://img.shields.io/crates/v/treeversal)](https://crates.io/crates/treeversal)
[![docs.rs](https://img.shields.io/docsrs/treeversal)](https://docs.rs/treeversal)

```
Customize your sandwich
├─Pick a bread (mandatory)
│  ├─(o) white
│  ├─( ) wheat
│  │  └─[ ] gluten-free wheat bread?
│  └─( ) rye
├─Pick a meat
│  ├─( ) ham
│  ├─( ) corned beef
│  ├─( ) turkey
│  └─( ) chicken
├─[ ] Pick vegetables
│  ├─[ ] lettuce
│  ├─[ ] tomato
│  ├─[ ] peppers
│  ├─[ ] onions
│  │  ├─(o) red onions
│  │  ├─( ) white onions
│  │  └─( ) grilled onions
│  └─[ ] avocado
├>[ ] Pick sauces
│  ├─[ ] mayonnaise
│  ├─[ ] barbeque sauce
│  └─[ ] oil and vinegar
└─{?} Finished?
```

A library for traversal and manipulation of a tree.
Create a tree of nodes as a `TreeDefinitionNode`, feed it to a `TreeInteractor`, and manipulate the tree by applying `TreeInteraction`s.

The tree definition, the tree interaction state, and the driver are all independent.
This crate comes with a driver for the terminal using the `console` crate (shown above),
but you could easily write your own for any GUI platform or input method.

## Unimplemented Features

- Create a `TreeInteractor` with preset defaults

## Nice to Have Features

- String input
- Ratatui driver
- Less ugly DSL
  - this may be impossible because this is rust not lisp
