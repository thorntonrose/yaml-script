# YAML Script

YAML Script is a YAML based scripting language. I created it as a fun, tongue-in-cheek way to learn Rust. (Who would want to use YAML for scripting?) It turned out to be kinda interesting, and I think I wouldn't hate it if I had to use it.


## Build

To build:

```
cargo {build | release}
```

To run tests:

```
cargo test [-- --nocapture]
```


## Run

To run:

```
./target/<debug|release>/ys <file>
```

Example:

```
./target/debug/ys examples/hello.ys
```


## Syntax

script:
```
<steps>
```

steps:
```
- <step>
...
```

step:
```
<var> | <echo> | <if> | <while> | <each> | <break> | <exec> | <def> | <call> | <exit>
```

(`<def>` and `<call>` are not yet implemented.)

var:
```
<name>: <boolean> | <integer> | <float> | <string> | <list> | <map> | <expression>
```

(Variables are global. `<list>` and `<map>` are not yet implemented.)

echo:
```
- echo: <expression>
```

if:
```
- if: <condition>
  [then: <steps>]
  [else: <steps>]
```

while:
```
- while: <condition>
  [do: <steps>]
```

each:
```
- each: <name>
  [in: <expression>]
  [do: <steps>]
```

break:
```
- break: <condition>
  [message: <string>]
```

exec:
```
- exec: <expression>
  [as: <name>]
```

def:
```
- def: <name>
  do: <steps>
```

call:
```
- call: <name>
  [with:
      <name>: <expression>
      ...]
```

exit:
```
- exit: <expression => number>
```

condition:
```
<expression> where true = true | non-zero | non-empty
```

expression:
```
${<name>} | ${<math-expression>} | ${<boolean-expression>}
```

(Expressions are handled by https://crates.io/crates/eval.)
