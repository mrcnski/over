<div align="center">
  <img src="mascot.svg" alt="OVER mascot — a parent record beside its child record, whose first field is the ^ parent marker" width="180"/>

# OVER

[![CI](https://github.com/mrcnski/over/actions/workflows/ci.yml/badge.svg)](https://github.com/mrcnski/over/actions/workflows/ci.yml)
[![crates.io](https://img.shields.io/crates/v/over.svg)](https://crates.io/crates/over)
[![Downloads](https://img.shields.io/crates/d/over.svg)](https://crates.io/crates/over)
[![Documentation](https://docs.rs/over/badge.svg)](https://docs.rs/over)
[![Issues](https://img.shields.io/github/issues-raw/mrcnski/over.svg)](https://github.com/mrcnski/over/issues)
[![License: MIT](https://img.shields.io/badge/License-MIT-yellow.svg)](https://opensource.org/licenses/MIT)

OVER: a data format similar to JSON with features such as variables, includes, and type safety.

</div>

## About

OVER is a general-purpose data format like XML or JSON, but much better. Here are some of its key features:

- OVER is designed to be intuitive for humans to read and write without sacrificing flexibility.
- It is powerful, with concepts such as variables, file includes, and object parents.
- It has an elegant and versatile type system which can safely represent all common data.
- It is resilient to errors by design and has no weird behavior or syntax like YAML or TOML.

## Example

A basic usage of OVER as a data format might look like this:

```
receipt: "Oz-Ware Purchase Invoice"
date:    "2012-08-06"
customer: {
    first_name:  "Dorothy"
    family_name: "Gale"
}

items: [
        {
         part_no:  "A4786"
         descrip:  "Water Bucket (Filled)"
         price:    01.47
         quantity: 4
        }
        {
         part_no:  "E1628"
         descrip:  "High Heeled \"Ruby\" Slippers"
         size:     8
         price:    133.70
         quantity: 1
        }
       ]

bill_to: {
    street:
    # A multi-line string. Can also be written as "123 Tornado Alley\nSuite16"
"123 Tornado Alley
Suite 16"
    city:  "East Centerville"
    state: "KS"
}

ship_to: bill_to

specialDelivery:
"Follow the Yellow Brick Road to the Emerald City. Pay no attention to the man behind the curtain."
```

This basic example already demonstrates a lot of nice features about OVER:

- You can immediately tell what kind of data each field contains.
- Multi-line strings require no special syntax; see `bill_to.street`.
- Variables; see `ship_to`.
- Comments (sounds simple, but JSON doesn't have them).

## Usage

Add OVER to your `Cargo.toml`:

```toml
[dependencies]
over = "*"
```

Rust code reading the above example data:

```rust
#[macro_use]
extern crate over;

use over::obj::Obj;

#[test]
fn example() {
    let obj = Obj::from_file("tests/test_files/example.over").unwrap();

    assert_eq!(obj.get("receipt").unwrap(), "Oz-Ware Purchase Invoice");
    assert_eq!(obj.get("date").unwrap(), "2012-08-06");
    assert_eq!(
        obj.get("customer").unwrap(),
        obj!{"first_name" => "Dorothy",
             "family_name" => "Gale"}
    );

    assert_eq!(
        obj.get("items").unwrap(),
        arr![
            obj!{"part_no" => "A4786",
                 "descrip" => "Water Bucket (Filled)",
                 "price" => frac!(147,100),
                 "quantity" => 4},
            obj!{"part_no" => "E1628",
                 "descrip" => "High Heeled \"Ruby\" Slippers",
                 "size" => 8,
                 "price" => frac!(1337,10),
                 "quantity" => 1},
        ]
    );

    assert_eq!(
        obj.get("bill_to").unwrap(),
        obj!{"street" => "123 Tornado Alley\nSuite 16",
             "city" => "East Centerville",
             "state" => "KS",
        }
    );

    assert_eq!(obj.get("ship_to").unwrap(), obj.get("bill_to").unwrap());

    assert_eq!(
        obj.get("specialDelivery").unwrap(),
        "Follow the Yellow Brick Road to the Emerald City. \
         Pay no attention to the man behind the curtain."
    );
}
```

Currently OVER has only been implemented for Rust; more languages may be supported in the future.

## Features

### Containers

OVER has three container types:

- An [array] where all elements must be of the same type of data (enforced by the parser).
- A (tuple) which can hold elements of different types.
- {Objects} which hold a map of fields to values of different types.

The following is a valid array, as each sub-tuple is the same type:

```
[ ("Alex" 10) ("Alice" 12) ]
```

The following is not a valid array. Can you see why?

```
[ ("Morgan" 13) ("Alan" 15 16) ]
```

### Variables

Fields defined in an object can be referenced later, but only in the same scope of the object:

```
var: 2

number: var

obj: {
    number: var # Invalid!
}
```

You can also define global variables. These are private and do not appear as fields in the final data:

```
@var: 2

number: @var

obj: {
    number: @var # Valid!
}
```

### Parents

An object can inherit the fields of another object. In the following example we define a template object called `@default` and define it to be the parent of `foo` and `bar` using the `^` field:

```
@default: {
    a: 1
    b: 2
}

foo: {
    ^: @default
    b: 5 # Override the value of "b" inherited from "@default".
    # foo.a == 1
}

bar: {
    ^: @default
    a: 5 # Override the value of "a" inherited from "@default".
    # bar.b == 2
}
```

### Object Field Access

The fields of an object can be accessed using dot notation. This is valid as long as the object in question is in scope and the field exists. Example:

```
obj: {
    sub_obj: {
        secret_value: 0
    }
}

value: obj.sub_obj.secret_value
```

This allows for some nice namespacing possibilities:

```
@colors: {
    red:   "#FF0000"
    green: "#00FF00"
    blue:  "#0000FF"
}

obj: {
    name: "Red monster"
    color: @colors.red
}
```

Arrays and tuples can also be indexed using dot notation:

```
tup: ("test" 0)

zero: tup.1
test: tup.zero
```

### Arithmetic on Values and Variables

Basic arithmetic is possible on values and variables. The available operators are `+`, `-`, `*`, `/`, and `%`, though not all operators can be applied to all types. The operators `*`, `/`, and `%` have a higher precedence than `+` and `-`.

Note that operators and their operands cannot be separated by spaces. The semantics of the language are such that a space after a value denotes the end of that value.

Here's an example:

```
grid: 16
x: 18 y: 20
width: 4
height: 6

rectangle: (x-x%grid y-y%grid width*grid height*grid)
```

### File Includes

In the spirit of modularity, OVER provides a facility for splitting up files. This functionality is best illustrated through an example.

Say we have two objects that we want in two separate files:

**`includes/obj1.over`:**

```
a: 1
b: 2
c: 3
```

**`includes/obj2.over`:**

```
a: 2
b: 3
c: 1
```

We can have something akin to a "main" file that contains the two files as sub-objects:

**`main.over`:**

```
obj1: <"includes/obj1.over">
obj2: <"includes/obj2.over">
```

The main benefits of includes are convenience and organization. We can also put arrays and tuples in separate files, which makes it easy to include, say, automatically-generated whitespace-delimited values. Finally, strings can also be in their own files, in which case they are parsed verbatim; no escaping of characters is done. This is a quite convenient option for large strings.

An example demonstrating inclusion of `Str`, `Arr`, and `Tup`:

**`main.over`:**

```
str: <Str "includes/str.over">
arr: <Arr "includes/arr.over">
tup: <Tup "includes/tup.over">
```

**`includes/str.over`:**

```
Multi-line string
which should be included verbatim
in another file. "Quotes" and $$$
don't need to be escaped.
```

**`includes/arr.over`:**

```
1 2 3 4 5
```

**`includes/tup.over`:**

```
1
"a"
3
"b"
5
```

Some notes about file includes:

- Global variables are not valid across files.
- Files cannot be included in a circular manner; e.g. if file `main` includes `sub-obj`, then `sub-obj` cannot include `main`.
- You can include the same file multiple times. File includes are only processed the first time they are encountered.
- Inclusion is only valid for `Obj`, `Str`, `Arr`, and `Tup`. When including an object file, the `Obj` keyword is optional.

### String Substitutions

Coming soon!

## Types

### Null

A simple null value, represented by `null`.

### Bool

`true`, or `false`. Take your pick.

### Int

An arbitrary-length signed integer type. Any token beginning with `-`, `+`, or a numeral will be either an `Int` or a `Frac` (see below).

**Examples:** `1`, `-2`, `+4`

### Frac

A sane representation of decimal values. Forget about float types and use fractions instead.

**Examples:** `-1/3`, `-5-1/4`, `2+1/2`, `42+6/1`

Fracs can also be written as decimals, which get converted automatically to fraction representation.

**Examples:** `2.5`, `-.0`

### Str

A unicode string type.

**Examples:** `"smörgåsbord"`, `"A string with \"quotes\""`

Multi-line strings are trivial:

```
"You don't need any
special syntax for multi-line strings;
newlines are captured automatically."
```

### Arr

An array container which can hold an arbitrary number of elements of a single type.

**Examples:** `[]`, `[1 2 3]`, `[(1 2) (3 4)]`

### Tup

A tuple container which can hold elements of different types.

**Examples:** `()`, `(1 "John")` `( ("x" 1/2) [1 2 3] )`

### Obj

The godfather of all types, the *object*. A hashmap of keys, which we call *fields*, to values, where a value can be of any type, including other objects.

Fields must be followed by a colon and cannot be one of the reserved keywords.

Reserved keywords:

- `@`
- `null`
- `true`
- `false`
- `Obj`
- `Str`
- `Arr`
- `Tup`

**Examples:**

`{ a: 1 b: 2 list: [a b b a] }`

`{ id: 4 field: { field: "Objects can be nested and each has their own scope." } }`

## Todo

As this project is being developed for my personal needs, there are some necessary steps to make it ready for `1.0` that I have little incentive to do myself. Any of the following would be a good way to contribute to the project:

- [Easy] Multi-line/block comments, e.g. `#[ ... ]#`. Should be able to nest these.
- [Hard] `super` keyword? i.e. `super.var` (disallow just `super`?). Not sure if this is worth the effort, but I can see potential use cases.
- [?] Write an Emacs mode, use JSON-mode as a starting point.
- [Easy] Benchmark against equivalent json files.
- [Medium] Implement string substitution.
- [Easy] Look through API guidelines: https://rust-lang-nursery.github.io/api-guidelines/checklist.html
- [Medium] Move error handling to Failure? https://www.reddit.com/r/rust/comments/7b88qp/failure_a_new_error_management_story/
- [Medium] Performance: Consider replacing HashMap internally. Use Flame to benchmark: https://github.com/TyOverby/flame

## What's wrong with JSON?

I started this project because I wanted an alternative to JSON. Why?

- The last element in a list cannot have a trailing comma.
- What's with the opening/closing braces?
- Floating-point numeric type.
- Arrays where different types are allowed.
- Field names have to be in quotes, e.g. `"name": "Johnny"` instead of `name: "Johnny"`. It's verbose and not ergonomic.
- No support for comments is a dealbreaker. Some JSON implementations allow them, but it's not standard.

JSON and other options are also lacking many of the features that I'm interested in, such as variables, the concept of object parents, file includes, and so on.

## What about YAML/others?

Let's compare the first example in this README ([Example](#example)) with the YAML version of the same data, taken from [Wikipedia](https://en.wikipedia.org/wiki/YAML#Example):

```yaml
---
receipt:     Oz-Ware Purchase Invoice
date:        2012-08-06
customer:
    first_name:   Dorothy
    family_name:  Gale

items:
    - part_no:   A4786
      descrip:   Water Bucket (Filled)
      price:     1.47
      quantity:  4

    - part_no:   E1628
      descrip:   High Heeled "Ruby" Slippers
      size:      8
      price:     133.7
      quantity:  1

bill-to:  &id001
    street: |
            123 Tornado Alley
            Suite 16
    city:   East Centerville
    state:  KS

ship-to:  *id001

specialDelivery:  >
    Follow the Yellow Brick
    Road to the Emerald City.
    Pay no attention to the
    man behind the curtain.
...
```

As you can see, this is much less clear than the OVER version. YAML has strange syntax (such as `&id001` and `*id001`; and what in the world are `>` and `|` supposed to be?) and a lack of useful syntax in others (every value looks like a string). Is the `date` field a number or a string? YAML is certainly more pleasing on a superficial level, which I suspect is the only reason it entered into general use, but it fails to stand up to some light scrutiny. It's all about looking nice while sacrificing clarity.

Look at [this answer](https://stackoverflow.com/a/18708156) on StackExchange for an example of how unintuitive YAML is. That's not the worst of it; there is a shocking amount of weirdness in the official spec. This design disaster also makes it impossible to write an efficient parser for YAML.

Finally, as seen throughout this README, OVER manages to be more powerful than YAML while being much simpler! This may strike you as a paradox, but it is just a consequence of the thoughtless design of YAML and company (don't think I've forgotten about TOML). There are options such as [StrictYAML](https://github.com/crdoconnor/strictyaml) but they are, in my opinion, just bandaids on a broken solution.
