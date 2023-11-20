# Thomas the parser engine

A PEG parser generator, but mostly an experiment to write parsers that
can be expressed at runtime that generate concrete syntax trees (as
opposed to to ASTs) and for which one can easily write tooling and
languages that can change their own syntax at runtime.

The idea is for this library to become the parser library I always
wished to have. We will see how far I can get before life distracts me
with other things.

**Note:** The project is named after Thomas Holloway the founder of
Royal Holloway, University of London.


## Ideas

- Reversible parsing: the parser generates the CST, from the CST it
  should be possible to regenerate the source code (character by
  character).

- The CST is immutable, so we change it by generating a new tree. It
  should be possible to generate a diff for each change so we can have
  powerful interactive modes in existing editors.

- (less precise) It should be possible to annotate the CST to provide
  semantic syntax highlighting.

## Tasks

- The parser should strive for efficiency and flexibility. I want to
  implement a packrat parser that supports left recursion following
  [this][packrat]

- Implement the rules to ignore whitespace

- ... A million more things


## References

[packrat] 1. Alessandro Warth, James R. Douglass, and Todd Millstein. 2008. Packrat
parsers can support left recursion. In Proceedings of the 2008 ACM
SIGPLAN symposium on Partial evaluation and semantics-based program
manipulation (PEPM '08). Association for Computing Machinery, New
York, NY, USA, 103–110. https://doi.org/10.1145/1328408.1328424
