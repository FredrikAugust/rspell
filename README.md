# CSpell alternative in rust

<!--toc:start-->

- [CSpell alternative in rust](#cspell-alternative-in-rust)
  - [Missing functionality](#missing-functionality)
  <!--toc:end-->

This is to practise systems programming

Currently not very functional, but with a _very_ naïve implementation and no
optimizations it seems promising to create something functional that matches or
beats the performance. Of course not a very fair comparison, but it's fun none
the less.

## Missing functionality

Will have to at some point introduce position information in the split words. I.e.

Parsing a file

```js
function tset() {}
```

Per now this will just complain that the file contains a typo with `tset` not
being a correct word, but it does not say _where_.
This will require rewriting the parser to handle the position, and propagate it
through the steps of splitting

- camel case
- snake case
- removing numbers
- etc.
