# Commands
Every command will be listed, with commas separating each alias

### `{`, `[`; `}`, `]`
Used to create blocks, put on either side of a block. They work like arrays and functions in other languages.
Blocks are basically mini stacks packed in values though blocks contain a list of both values and commands.

Here is an example of a simple block: `{2+}`
When it is called with the `()` operator, `2` will be added to the stack and then `+` will be run.
It is therefore basically a function that adds 2 to the value on top of the stack.

Blocks can also be used as simple lists of values, that will all be added to the stack when called.

Blocks can also be manipulated with, with the `\/`, `.` and `len` commands.

### `inc`, `include`
Pops a string from the stack and runs the stalch code contained in the file on
the relative path represented in the string. This is used many of the example
files to include the stdlib.stalch file.

### `pack`, `@`
Takes the whole stack and puts it into a block.

#### Example
Imagine the stack `A B C D`, running `pack` would result in the stack becoming:
`{A B C D}`. **NOTE**: that the size of the stack is now 1, no matter the size
of the stack before.

### `size`
Pops the current length or size of the stack as an Integer to the stack.

### `len`
If the last value on the stack is a block, the length of the block will be pushed
to the stack as an Integer. Otherwise `null` will be pushed.

### `dup`, `d`
Pops a copy of the last element of the stack onto the stack.

Example: `A B C` -> `A B C C`

#### Error
Throws `StackEmpty` if the stack is empty when called

### `not`, `!`
Runs a bitwise not on Bools and Integers: `false` -> `true`, `0` -> `-1`.

Floats and Strings are casted to Bools before running bitwise not (see `bool`).
Blocks and Null become `null`.

### `if`, `?`
Pops three values from and pushes either the second value or third value
depending on whether the first value evaluates to `true` or `false` respectively.

Imagine the following stack: `A B C`. If `A` evaluates to `true`, `B` will be and,
if it on the other hand evaluates to `false` it will be `C`

#### Examples
- `true 1 2 ?` returns `1`

- `false 1 2 ?` returns `2`

- `false {"That was true" _} {"That was false" _}?()` prints "That was false".

#### Error
Throws `StackEmpty` if the stack is smaller than 3

### `def`, `:=`
Pops two values from the stack. If only of them is a variable name, the other
value, will be attached to that name. Otherwise prioritises the last (the right-most)
as the name to assign to.

#### Examples
Running `4 four :=` `four` will now run `4` when called. `four 4 :=` will do the
same if and only if `four` hasn't been assigned to before. The variable name needs
to be the last value to re-assign a variable. This is often used to create custom
functions, although since those will be defined as blocks, they have to be called
with `()`

#### Errors
Throws `StackEmpty` if the stack is smaller than 2.

Throws `InvalidAssignArg` if there is no variable name in the two values.

### `apply`, `()`
Runs the last block on the stack. If the block only consists of values, this will
simply push all the values onto the stack.

If run on a string, it does nothing. This is allow to using `.`, `;`, `\/` generically.

#### Errors
Throws `StackEmpty` if the stack is empty.

Throws `InvalidApplyArg`, if the last value on the stack isn't a block or a string.

### `read`, `<-`
Reads a line from STDIN and pushes as a string to the stack. The string will be trimmed.

### `swap`, `$`
Swaps the place of the two last values on the stack.
#### Errors
Throws `StackEmpty` if the stack is empty.

### `split`, `\/`, `\\/`
Pops an integer, and a block or string from the stack.
Pushes the two blocks/strings split at the given index.

#### Examples
- `{A B C D} 3 split` becomes `{A} {B C D}`
- `"hello" 3 split` becomes `"he" "llo"`
- `{} 0 split` becomes `{} {}`
- `"" 0 split` becomes `"" ""`

#### Errors
Throws `StackEmpty` if smaller than 2.

Throws `InvalidSplitArg` if the second-last element is neither a block nor string, or
if the last value is not an Integer.

Throws `OutOfBounds` if the index Integer is larger than the length of the
block/string or if it's negative.

### `get`, `.`
Takes a value out of a block or string and pushes it as its own block/string.
#### Examples
- `{3 2 1 0} 2 get` becomes `{3 1 0} {2}`
- `"hello" 4 get` becomes `"ello" "h"`

#### Errors
Throws `StackEmpty` if smaller than 2.

Throws `OutOfBounds` if index is not smaller than the length.
### `dupget`, `;`
Like `get`, it takes a value out of a block or string and pushes it as its own block/string.
Although the value also stays in its place in the block/string.
#### Examples
- `{3 2 1 0} 2 dupget` becomes `{3 2 1 0} {2}`
- `"hello" 4 dupget` becomes `"hello" "h"`

#### Errors
Throws `StackEmpty` if smaller than 2.

Throws `OutOfBounds` if index is not smaller than the length.
### `move`, `<>`
Pops an Integer and moves the next, last value of the stack that many places back.
#### Example
`A B C D 2 move` becomes `A D B C`
#### Errors
Throws `StackEmpty` if stack is smaller than 2.

Throws `OutOfBounds` if the value is to move out of the stack.
### `grab`, `#`
Pops an Integer and moves a value that many values back to the last position.

`grab` and `move` are opposites, so `n move n grab` and `n grab n move` should
leave the stack unaffected.
#### Example
`A B C D 2 grab` becomes `A C D B`
#### Errors
Throws `StackEmpty` if stack is empty.

Throws `OutOfBounds` if there is no value at that index.
### `dupgrab`, `:`
Pops an Integer and copies a value that many values back to the last position.

Like `grab` but instead of moving the value, it lets it stay there and just
pushes a copy.
#### Example
`A B C D 2 dupgrab` becomes `A B C D B`
#### Errors
Throws `StackEmpty` if stack is empty.

Throws `OutOfBounds` if there is no value at that index.
### `drop`, `~`
Removes the last value from the stack.
#### Errors
Throws `StackEmpty` if stack is empty.
### `type`, `t`
Pops the last value on the stack and pushes its type as a String.
See following table:

| Type          | Output string |
|:------------- |:-------------:|
| String        |     "str"     |
| Bool          |     "bool"    |
| Integer       |     "int"     |
| Float         |    "float"    |
| Block         |    "block"    |
| Null          |    "null"     |
| Variable name |     "var"     |
#### Errors
Throws `StackEmpty` if stack is empty.

### `float`, `f`
Casts the last value in the stack to a Float
#### Errors
Throws `StackEmpty` if stack is empty.
### `int`, `i`
Casts the last value in the stack to an Integer
#### Errors
Throws `StackEmpty` if stack is empty.
### `bool`, `b`
Casts the last value in the stack to a Bool
#### Errors
Throws `StackEmpty` if stack is empty.
### `eq`, `==`
Equality operator
#### Errors
Throws `StackEmpty` if smaller than 2.
### `neq`, `!=`
Inequality operator
#### Errors
Throws `StackEmpty` if smaller than 2.
### `>`
Greater than operator
#### Errors
Throws `StackEmpty` if smaller than 2.
### `>=`
Greater than or equals operator
#### Errors
Throws `StackEmpty` if smaller than 2.
### `<`
Less than operator
#### Errors
Throws `StackEmpty` if smaller than 2.
### `<=`
Less than or equals operator
#### Errors
Throws `StackEmpty` if smaller than 2.
### `wrte`, `->`
Writes last value in the stack to STDOUT with**out** an appended newline.
#### Errors
Throws `StackEmpty` if stack is empty.
### `prnt`, `_`
Writes last value in the stack to STDOUT **with** an appended newline.
#### Errors
Throws `StackEmpty` if stack is empty.
### `exit`, `x`
If run in a stack, stops running current stack. If outside, quits the current
program.

**NOTE**: The interpreter runs each line by its own, so `exit` will only exit
that line, it won't close the interpreter. You `$exit` or Ctrl+C to do that.
### `or`, `|`
Bitwise or
#### Errors
Throws `StackEmpty` if smaller than 2.
### `and`, `&`
Bitwise and
#### Errors
Throws `StackEmpty` if smaller than 2.
### `xor`, `^`
Bitwise xor
#### Errors
Throws `StackEmpty` if smaller than 2.
### `add`, `+`
Addition
#### Errors
Throws `StackEmpty` if smaller than 2.
### `sub`, `-`
Subtraction
#### Errors
Throws `StackEmpty` if smaller than 2.
### `mul`, `*`
Multiplication
#### Errors
Throws `StackEmpty` if smaller than 2.
### `pow`, `**`
Calculates the last next-to-last value in the stack to power of the last value
in the stack and pushes that.

`a n **` becomes aⁿ.
#### Errors
Throws `StackEmpty` if smaller than 2.
### `div`, `/`
Division
#### Errors
Throws `StackEmpty` if smaller than 2.
### `rem`, `%`
Modulus. Calculates the remainder of a division of two numbers.
#### Errors
Throws `StackEmpty` if smaller than 2.
