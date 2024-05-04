# platelet

`platelet` is a tiny HTML templating library, designed for composing small **server-rendered** "components".

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

`{{variable}}` inserts a (sanitized) string.

If the variable is not defined then an error is returned.

| Data type | Rendered as   |
| --------- | ------------- |
| Number    | A number      |
| String    | A string      |
| Boolean   | true or false |
| Null      | blank         |
| Array     | error         |
| Object    | error         |

HTML Attributes starting with a `pl-` are special. They are inspired by Vue's directives.

| attribute       | behaviour                                        |
| --------------- | ------------------------------------------------ |
| `pl-if`         | render this element if the condition is truthy   |
| `pl-else-if`    | following a `pl-if`                              |
| `pl-else`       | following a `pl-if` or `pl-else-if`              |
| `pl-for`        | render element multiple times, see details below |
| `pl-inner-html` | set the innerHTML without sanitization           |
| `pl-outer-html` | set the outerHTML without sanitization           |
| `pl-class`      | conditionally set css classes                    |

| attribute | behaviour                                                        |
| --------- | ---------------------------------------------------------------- |
| `pl-src`  | renders the template at the path and replaces the content        |
| `pl-data` | only makes sense when used with `pl-src`, pass json to the child |
| `pl-slot` | marks the component as a slot - one per document                 |
| `pl-is`   | when passed a string                                             |

`pl-for` allows 4 types of expression:

```html
<div pl-for="item in items">{{item.text}}</div>
<div pl-for="(item, index) in items">...</div>
<div pl-for="(value, key) in object">...</div>
<div pl-for="(value, name, index) in object">...</div>
```

Template

```html
<slot pl-src="./sidebar.html" pl-data='{"username": data.username}'>
  <p>Some text...</p>
</slot>
```

## Expressions

On anything: `==`, `!=`

On numbers: `>`, `<`, `>=`, `<=`

On booleans: `&&`, `||`, `!`

On arrays: `len([...])`

On objects: `size([...])`

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
