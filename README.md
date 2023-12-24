# SIC/XE Assembler

Yet another SIC/XE assembler.

## Features

- Support SIC/XE instructions and directives.
- Support control sections.
- Support program blocks.
- Support literals.
- Support symbol-defining directive (`EQU`).
- Support syntax checking.
- Support basic semantic checking.

## Usage

You'll need the latest stable version of Rust to build the assembler.

```bash
$ cargo build --release
$ ./target/release/sicxe-cli <source-file>
```

## Architecture

The assembler is generally divided into 4 parts:

- Tokenizer (Just remove comments and split the line into tokens)
- Parser (Parse the tokens into **Frames**)
- Transformer(s) (Transform a sequence of Frames into another sequence of Frames while resolving directives and symbols)
- Optimizer (Optimize the object code layout)

### Frames

There are 3 types of frames:

```rs
pub enum FrameInner {
    Instruction(Instruction),
    Directive(Directive),
    ObjectRecord(ObjectRecord),
}
```

- Instruction: Represents an instruction (e.g. `LDA`, `J`, `JSUB`, etc.)
- Directive: Represents a directive (e.g. `START`, `END`, `BYTE`, `WORD`, etc.)
- ObjectRecord: Represents an object record (e.g. `T`, `M`, `E`, etc.)

### Transformer(s)

- Section Splitter: It split the source program into control sections, and treats each section a separate program that can be passed to other transformers.
- Block Rearranger: It rearranges the frames' layout based on the program blocks configuration.
- Literal Dumper: It dumps the literals into `BYTE` directives.
- Symbol Resolver: It resolves the symbols and replaces them with their addresses.
- Translator: It translates the frames into object records.

### Optimizer

It optimizes the object code layout to make it compact as possible.

## Copyright & License

© 2023 Jacob Lin

Distributed under the AGPL-3.0 License. See `LICENSE` for more information.
