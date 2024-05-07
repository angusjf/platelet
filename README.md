# platelet

WARNING: This project is an unreleased work in progress!

`platelet` is a tiny HTML-first templating library.

It sits somewhere between the simplitcity of 'moustache' templates and JSX/frontend framework templating libraries.

Notable features:

- HTML specific
  - All logic is done with HTML attributes
  - Why? The templates are valid HTML, which means you can bring your own formatters & tooling
- JSON-based
  - Why? Keeps templates portable
- Support for importing templates
  - Why? Everyone loves components
- Script and stylesheet de-duplicicating
  - Why? Allows for co-location of styles, scripts and templates, even when the templates are rendered more than once

The philosophy of platelet is that rendering logic

# Syntax

## Text Nodes

In an HTML text node, `{{variable}}` inserts a (sanitized) string.

```html
<h1>Welcome back {{user.name}}!</h1>
```

<details>
<summary>
**more details on text nodes**
</summary>

If the variable is not defined then an error is returned.

| Data type | Rendered as   |
| --------- | ------------- |
| Number    | A number      |
| String    | A string      |
| Boolean   | true or false |
| Null      | blank         |
| Array     | error         |
| Object    | error         |

</details>

## HTML Attributes

In an HTML attribute, prefixing the attribute with `^` allows you to set the value to a `platelet` expression.

```html
<a ^href='"/products/" + slug'></a>
```

If the expression is **falsy**, the attribute will not render.

The `^class` attribute is a special case, as it also accepts a map.
It can be used in combination with the regular `class` attribute.

```html
<div
  class="static"
  :class="{ active: isActive, 'text-danger': hasError }"
></div>
```

This will render:

```html
<div class="static active text-danger"></div>
```

When

## `pl-` attributes

HTML Attributes starting with a `pl-` are special. They are inspired by Vue's directives.

| attribute    |
| ------------ |
| `pl-if`      |
| `pl-else-if` |
| `pl-else`    |
| `pl-for`     |
| `pl-html`    |
| `pl-attrs`   |
| `pl-src`     |
| `pl-data`    |
| `pl-slot`    |
| `pl-is`      |

<details>
<summary>Here's a detailed breakdown of what they do </summary>

### Conditinals: `pl-if`, `pl-else-if`, `pl-else`

`pl-if` will only render this element if the condition is truthy

`pl-else-if`, used following a `pl-if`, will only render this element if the condition is truthy

`pl-else`, used following a `pl-if` or `pl-else-if`, will render this element otherwise

### `pl-for`

render element multiple times

allows 4 types of expression:

```html
<div pl-for="item in items">{{item.text}}</div>
<div pl-for="(item, index) in items">...</div>
<div pl-for="(value, key) in object">...</div>
<div pl-for="(value, name, index) in object">...</div>
```

### `pl-html`

set the innerHTML without sanitization

to set the outerHTML without sanitization, apply this to a `<template>`

### `pl-attrs`

conditionally set html attributes, using a (flat) json object

for example:

```html
<h1 pl-attrs='{ "x": "y" }'></h1>
```

### `pl-src`

given a path as a string, renders the template at the path and replaces the element

```html
<slot pl-src="./sidebar.html" pl-data='{"username": data.username}'>
  <p>Some text...</p>
</slot>
```

### `pl-data`

only makes sense when used with `pl-src`, pass json to the child
any expression returning an object or list of objects, in which case objects are merged

### `pl-slot`

marks the component as a slot - one per document
no value to be supplied

### `pl-is`

replace the rendered element's tag with this element, given an expression that returns a string

Compatibility matrix

| attribute         | `pl-if` , `pl-else-if` , `pl-else` | `pl-for` | `pl-inner-html` , `pl-outer-html` | `pl-attrs`   | `pl-src`   | `pl-data`   | `pl-slot`   | `pl-is`   |
| ----------------- | ---------------------------------- | -------- | --------------------------------- | ------------ | ---------- | ----------- | ----------- | --------- |
| `pl-if ...`,      |                                    | 𐄂        | 𐄂                                 | ----------   | --------   | ---------   | ---------   | -------   |
| `pl-for`          | 𐄂                                  |          | 𐄂                                 | ----------   | --------   | ---------   | ---------   | -------   |
| `pl-outer-html..` | 𐄂                                  | 𐄂        |                                   | ----------   | --------   | ---------   | ---------   | -------   |
| `pl-attrs`        | 𐄂                                  | 𐄂        | (outer yes, inner no)             | ❌---------- | --------   | ---------   | ---------   | -------   |
| `pl-src`          | 𐄂                                  | 𐄂        |                                   | ----------   | ❌-------- | ---------   | ---------   | -------   |
| `pl-data`         | 𐄂                                  | 𐄂        |                                   | ----------   | --------   | ❌--------- | ---------   | -------   |
| `pl-slot`         |                                    |          |                                   | ----------   | --------   | ---------   | ❌--------- | -------   |
| `pl-is`           | 𐄂                                  | 𐄂        | 𐄂                                 | ----------   | --------   | ---------   | ---------   | ❌------- |

</details>

`<template>` elements have special meaning when `pl-` attributes are

## Expressions

On anything: `==`, `!=`, `&&`, `||`, `!`, `x ? y : z`

On numbers and strings: `+` (addition or concatenation)

On numbers: `-`, `*`, `/`, `%` (mod)

On numbers: `>`, `<`, `>=`, `<=`

On objects and arrays, indexing operator `a[b]`

On arrays: `len(z)`

Expressions can be bracketed `(9 + 3) / 2 == 6`

## Truthiness

`false`, `[]`, `""`, `{}`, `null` are **falsy**.

All other values are **truthy**.

# Example

Imagine a directory, `templates` containing these files:

`templates/index.html`

```html
<!doctype html>
<html>
  <head>
    <title>{data.title}</title>
    <style>
      :root {
        font-family: sans-serif;
      }
    </style>
  </head>
  <body>
    <h1>{data.title}</h1>
    <slot pl-src="./blogpost.html" pl-for="b in blogposts" pl-data="b"></slot>
  </body>
</html>
```

`templates/blogpost.html`

```html
<article>
    <img src="{img_url}">
    <div>
        <h2>
            <a href="{link}">{title}</a>
        <h2>
        <slot pl-outer-html="summary"></slot>
        <date>{date}</date>
    </div>
</article>
<style>
    article {
        display: flex;
    }
</style>
```

And the following JSON file:

`variables.json`

```json
{
  "data": {
    "title": "Angus' Blog",
    "blogposts": [
      {
        "img_url": "...",
        "link": "...",
        "summary": "...",
        "title": "...",
        "date": "01/11/2025"
      },
      {
        "img_url": "...",
        "link": "...",
        "summary": "...",
        "title": "...",
        "date": "01/11/2020"
      }
    ]
  }
}
```

Running this script:

```bash
cat variables.json | platelet templates/index.html
```

Or using the api:

```rust
let args: Value = ...;
let template: &str = "...";
Platelet::render(args, template)
```

Will produce a string output as expected.

<details>
<summary>
Here's the output if you're interested
</summary>
```html
TODO
```
<details>

# Limitations

`platelet` does not allow templating for CSS and JS files, other than the ability to insert

This is intentional as
