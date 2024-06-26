# platelet

`platelet` is an HTML-first templating language.

This repo contains a Rust library for rendering `platelet` templates.

## Why platelet?

Unlike `moustache`, `handlebars`, `Jinja`, `Liquid` and other templating languages, `platelet`'s syntax is part of HTML (similar to Vue.js).

This has a few upsides:

- **Higher level** but [less powerful](https://www.w3.org/DesignIssues/Principles.html#:~:text=Principle%20of%20Least%20Power) than direct string manipulation
- The language is **natural to read and write** when working with HTML, and control flow follows HTML structure
- You can use your own HTML formatters and **tooling**
- HTML **sanitization** is more natural and straightforward

## Example

You can explore live examples in the [platelet playground](https://angusjf.com/platelet/playground)

###### Template

```html
<ul pl-if="n > 0">
  <li pl-for="i in [1, 2, 3]">{{ i }} × {{ n }} = {{ i * n }}</li>
</ul>
```

###### Context (input)

```json
{ "n": 7 }
```

###### Output

```html
<ul>
  <li>1 × 7 = 7</li>
  <li>2 × 7 = 14</li>
  <li>3 × 7 = 21</li>
</ul>
```

<details>
<summary>More examples</summary>

### Advanced example

###### Template `templates/index.html`

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

###### Template `templates/blogpost.html`

```html
<article>
  <img ^src="blogpost.img_url" />
  <div>
    <h2>
      <a ^href="blogpost.link">{{blogpost.title}}</a>
    </h2>
    <template pl-html="blogpost.summary"></template>
    <date>{{blogpost.date}}</date>
  </div>
</article>
<style>
  article {
    display: flex;
  }
</style>
```

###### Context (input)

```json
{
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
```

</details>

## Reference

| Syntax            | Example               | Details              |
| ----------------- | --------------------- | -------------------- |
| `pl-` directives  | `pl-if`, `pl-for` ... | [→](#pl--directives) |
| `^` attributes    | `^class`, `^name` ... | [→](#-attributes)    |
| `{{ ... }}` nodes | `{{ user.email }}`    | [→](#text-nodes)     |
| Expressions       | `1 + users[i].score`  | [→](#expressions)    |

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
| `pl-slot`    |
| `pl-is`      |

### Conditionals: `pl-if`, `pl-else-if`, `pl-else`

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

The attributes set on the element (regular attributes or rendered `^` attributes) are used as the context for rendering the template.

### `pl-slot`

On a `<slot>`, `pl-slot` (with an optional name) marks the element as a slot, to be replaced.

On a `<template>`, `pl-slot` marks the template content as "what fills that slot".

###### `index.html`

```html
<slot pl-src="layout.html">
    <template pl-slot="sidebar">
        <ul> ...
    </template>
    <template pl-slot="content">
        <table> ...
    </template>
</slot>
```

###### `layout.html`

```html
<body>
  <nav>
    <slot pl-slot="sidebar"></slot> </nav>
  <main>
    <slot pl-slot="content"></slot>
  </main>
</body>
```

###### Output

```html
<body>
  <nav>
    <ul> ...
  </nav>
  <main>
    <table> ...
  </main>
</body>
```

### `pl-is`

Replace the rendered element's tag with this element, given an expression that returns a string

```html
<slot pl-is='i == 0 ? "h1" : "h2"'>{item}</slot>
```

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

## Text Nodes

In an HTML text node, `{{variable}}` inserts a (sanitized) string.

```html
<h1>Welcome back {{user.name}}!</h1>
```

If the variable is not defined, or a key does not exist on an object, null is returned.

| Data type | Rendered as   |
| --------- | ------------- |
| Number    | A number      |
| String    | A string      |
| Boolean   | true or false |
| Null      | error         |
| Array     | error         |
| Object    | error         |

## Expressions

All valid JSON values are valid `platelet` expressions. On top of this, single-quoted strings `'like this'` are allowed for convenience when working with HTML.

### Operators

On anything: `==`, `!=`, `&&`, `||`, `!`, `x ? y : z`

On numbers: `+` (addition)
On strings and arrays: `+` (concatenation)
On objects: `+` (shallow merge, right hand side overriding)

On numbers: `-`, `*`, `/`, `%` (mod)

On numbers: `>`, `<`, `>=`, `<=`

On objects arrays and strings, indexing operator `a[b]`

On objects, dot access: `{"name": "angus"}.name`

On arrays, objects and strings: `len(z)`

Expressions can be bracketed `(9 + 3) / 2 == 6`

### Truthiness

`false`, `[]`, `""`, `{}`, `null` are **falsy**.

All other values are **truthy**.
