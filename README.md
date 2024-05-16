# platelet

WARNING: This project is an unreleased work in progress!

`platelet` is an HTML-first templating language.

This repo contains a Rust library for rendering `platelet` templates.

# Why platelet?

Unlike `moustache`, `handlebars`, `Jinja`, `Liquid` and other templating languages, `platelet`'s syntax is part of HTML.

# Examples

<details open>
<summary>Simple example</summary>

```html
TODO
```

</details>

<details>
<summary>
Advanced example
</summary>
Imagine a directory, `templates` containing these files:

`templates/index.html`

```html
<!doctype html>
<html>
  <head>
    <title>{{ title }}</title>
  </head>
  <body>
    <template pl-for="b in blogposts" pl-src="./blogpost.html" ^blogpost="b">
    </template>
  </body>
</html>
```

`templates/blogpost.html`

```html
<article>
    <img src="{blogpost.img_url}">
    <div>
        <h2>
            <a href="{blogpost.link}">{title}</a>
        <h2>
        <slot pl-outer-html="blogpost.summary"></slot>
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

</details>

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

# Reference

| Syntax            | Example               | Details                    |
| ----------------- | --------------------- | -------------------------- |
| `pl-` directives  | `pl-if`, `pl-for` ... | [details](#pl--directives) |
| `^` attributes    | `^class`, `^name` ... | [details](#^-attributes)   |
| `{{ ... }}` nodes | `{{ user.email }}`    | [details](#text-nodes)     |
| Expressions       | `1 + users[i].score`  | [details](#expressions)    |

## `^` Attributes

In an HTML attribute, prefixing the attribute with `^` allows you to set the value to a `platelet` expression.

```html
<a ^href='"/products/" + slug'></a>
```

If the expression is `false` or `null`, the attribute will not render.

```html
<div
  class="static"
  ^class="{ active: isActive, 'text-danger': hasError }"
  ^name="null"
></div>
```

This will render:

```html
<div class="static active text-danger"></div>
```

## `pl-` Directives

HTML Attributes starting with a `pl-` are special. They are inspired by Vue's directives.

| attribute    |
| ------------ |
| `pl-if`      |
| `pl-else-if` |
| `pl-else`    |
| `pl-for`     |
| `pl-html`    |
| `pl-src`     |
| `pl-data`    |
| `pl-slot`    |
| `pl-is`      |

<details>
<summary>Here's a detailed breakdown of what they do </summary>

### Conditinals: `pl-if`, `pl-else-if`, `pl-else`

`pl-if` will only render this element if the expression is truthy

`pl-else-if`, used following a `pl-if`, will only render this element if the expression is truthy

`pl-else`, used following a `pl-if` or `pl-else-if`, will render this element otherwise

```html
<button pl-if="stock >= 1">Add to cart</button>
<button pl-else-if="stock > 0">Add to cart (1 item left!)</button>
<button pl-else disabled>Out of stock</button>
```

If applied to a `<template>`, the template will be and the children rendered.

### `pl-for`

Render element multiple times.

Allows 4 types of expression:

```html
<div pl-for="item in items">{{item.text}}</div>
<div pl-for="(item, index) in items">...</div>
<div pl-for="(value, key) in object">...</div>
<div pl-for="(value, name, index) in object">...</div>
```

If applied to a `<template>`, the template will be removed and the children rendered.

### `pl-html`

Set the innerHTML (without sanitization) to the given expression.

To set the outerHTML, apply this to a `<template>`.

```html
<p pl-html="markdown"></p>
```

```json
{ "markdown": "<h1>Content from a CMS</h1>..." }
```

### `pl-src`

Given a path as a string, renders the template at the path and replaces the element.

```html
<slot pl-src="./sidebar.html" ^username="data.username">
  <p>Some text...</p>
</slot>
```

The attributes set on the element (regular attributes or rendered `^` atributues) are used as the context for rendering the template.

### `pl-slot`

Marks the component as a slot.
Option

### `pl-is`

Replace the rendered element's tag with this element, given an expression that returns a string

```html
<slot pl-is='i == 0 ? "h1" : "h2"'>{item}</slot>
```

</details>

# Expressions

All valid JSON values are valid `platelet` expressions. On top of this, single-quoted strings `'like this'` are allowed for convinience when working with HTML.

## Operators

On anything: `==`, `!=`, `&&`, `||`, `!`, `x ? y : z`

On numbers: `+` (addition)
On strings and arrays: `+` (concatenation)
On objects: `+` (shallow merge, right hand side overriding)

On numbers: `-`, `*`, `/`, `%` (mod)

On numbers: `>`, `<`, `>=`, `<=`

On objects arrays and strings, indexing operator `a[b]`

On objects, dot access: `{"name": "angus"}.name`

On arrays: `len(z)`

Expressions can be bracketed `(9 + 3) / 2 == 6`

# Truthiness

`false`, `[]`, `""`, `{}`, `null` are **falsy**.

All other values are **truthy**.

# Text Nodes

In an HTML text node, `{{variable}}` inserts a (sanitized) string.

```html
<h1>Welcome back {{user.name}}!</h1>
```

If the variable is not defined then an error is returned.

| Data type | Rendered as   |
| --------- | ------------- |
| Number    | A number      |
| String    | A string      |
| Boolean   | true or false |
| Null      | blank         |
| Array     | error         |
| Object    | error         |
