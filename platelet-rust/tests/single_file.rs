use platelet::{
    render,
    renderer::{RenderError, RenderErrorKind},
};
use serde_json::{json, Map};

#[test]
fn templateless_text_node() {
    let vars = json!({ "hello": "world" });

    let result = render("<h1>nothing here</h1>".into(), &vars);
    assert_eq!(result.unwrap(), "<h1>nothing here</h1>");
}

#[test]
fn templateless_html_doc() {
    let vars = json!({ "hello": "world" });

    let result = render(
        "<!doctype html><html><head><title>a</title></head><body></body></html>".into(),
        &vars,
    );
    assert_eq!(
        result.unwrap(),
        "<!DOCTYPE html><html><head><title>a</title></head><body></body></html>"
    );
}

#[test]
fn templateess_table_fragment() {
    let vars = Map::new().into();

    let result = render(
        "<tr><td>Data 0</td><td>Value 0</td></tr>\
         <tr><td>Data 1</td><td>Value 1</td></tr>"
            .into(),
        &vars,
    );
    assert_eq!(
        result.unwrap(),
        "<tr><td>Data 0</td><td>Value 0</td></tr>\
         <tr><td>Data 1</td><td>Value 1</td></tr>"
    );
}

#[test]
fn templated_text_node() {
    let vars = json!({ "hello": "world" });

    let result = render("<h1>{{hello}}</h1>".into(), &vars);
    assert_eq!(result.unwrap(), "<h1>world</h1>");
}

#[test]
fn complex_text_node() {
    let vars = json!({ "user": {"firstname": "Yuri", "lastname" : "Gagarin" } });

    let result = render(
        "<h1>Dear {{user.firstname}} {{user.lastname}},</h1>".into(),
        &vars,
    );
    assert_eq!(result.unwrap(), "<h1>Dear Yuri Gagarin,</h1>");
}

#[test]
fn text_node_with_expressions() {
    let vars = json!({ "countries": [ "portugal" ] });

    let result = render("<h1>{{countries[0]}} {{ 1 + 2 }}</h1>".into(), &vars);
    assert_eq!(result.unwrap(), "<h1>portugal 3</h1>");
}

#[test]
fn pl_if() {
    let vars = Map::new().into();

    let result = render(
        "<p>this</p>\
                    <p pl-if='false'>not this</p>\
                    <p>also this</p>"
            .into(),
        &vars,
    );
    assert_eq!(result.unwrap(), "<p>this</p><p>also this</p>");
}

#[test]
fn pl_else_if_true() {
    let vars = Map::new().into();

    let result = render(
        "<p>this</p>\
                        <p pl-if='false'>not this</p>\
                        <p pl-else-if='true'>also this</p>"
            .into(),
        &vars,
    );
    assert_eq!(result.unwrap(), "<p>this</p><p>also this</p>");
}

#[test]
fn pl_else_if_false() {
    let vars = Map::new().into();

    let result = render(
        "<p>this</p>\
                        <p pl-if='false'>not this</p>\
                        <p pl-else-if='false'>also this</p>"
            .into(),
        &vars,
    );
    assert_eq!(result.unwrap(), "<p>this</p>");
}

#[test]
fn pl_is() {
    let vars = json!({ "header": true });

    let result = render("<p pl-is='header ? \"h1\" : \"h2\"'>this</p>".into(), &vars);
    assert_eq!(result.unwrap(), "<h1>this</h1>");
}

#[test]
fn pl_html() {
    let vars = json!({ "content": "<p>hello world</p>" });

    let result = render(
        "<article pl-html='content'>something that used to be here</article>".into(),
        &vars,
    );
    assert_eq!(result.unwrap(), "<article><p>hello world</p></article>");
}

#[test]
fn pl_html_with_vars_are_not_rendered() {
    let vars = json!({ "content": "<p>hello {{mistake}} world</p>" });

    let result = render(
        "<article pl-html='content'>something that used to be here</article>".into(),
        &vars,
    );
    assert_eq!(
        result.unwrap(),
        "<article><p>hello {{mistake}} world</p></article>"
    );
}

#[test]
fn pl_html_with_template() {
    let vars = json!({ "content": "<p>hello world</p>" });

    let result = render(
        "<template pl-html='content'>something that used to be here</template>".into(),
        &vars,
    );
    assert_eq!(result.unwrap(), "<p>hello world</p>");
}

#[test]
fn template_preserved() {
    let vars = Map::new().into();

    let result = render("<template><h1>hello</h1></template>".into(), &vars);
    assert_eq!(result.unwrap(), "<template><h1>hello</h1></template>");
}

#[test]
fn pl_for() {
    let vars = Map::new().into();

    let result = render(
        "<div><p pl-for='x in [1,2,3]'>{{x}}</p></div>".into(),
        &vars,
    );
    assert_eq!(result.unwrap(), "<div><p>1</p><p>2</p><p>3</p></div>");
}

#[test]
fn pl_for_template() {
    let vars = Map::new().into();

    let result = render(
        "<div><template pl-for='x in [1,2,3]'><p>{{x}}</p></template></div>".into(),
        &vars,
    );
    assert_eq!(result.unwrap(), "<div><p>1</p><p>2</p><p>3</p></div>");
}

#[test]
fn pl_if_template() {
    let vars = Map::new().into();

    let result = render(
        "<div><template pl-if='[1]'><p>hello</p><p>world</p></template></div>".into(),
        &vars,
    );
    assert_eq!(result.unwrap(), "<div><p>hello</p><p>world</p></div>");
}

#[test]
fn loop_with_if_else() {
    let vars = Map::new().into();

    let result = render(
        "<div pl-if='\"A\" == \"Z\"'>A</div>\
                         <div pl-for='_ in [1,3]' pl-else-if='\"A\" == \"A\"'>B</div>\
                         <div pl-else-if='\"A\" == \"A\"'>C</div>\
                         <div pl-else>Not A/B/C</div>"
            .into(),
        &vars,
    );
    assert_eq!(result.unwrap(), "<div>B</div><div>B</div>");
}

#[test]
fn pl_else_true() {
    let vars = Map::new().into();

    let result = render(r#"<p pl-if="true">A</p><p pl-else>B</p>"#.into(), &vars);
    assert_eq!(result.unwrap(), "<p>A</p>");
}

#[test]
fn pl_else_false() {
    let vars = Map::new().into();

    let result = render(r#"<p pl-if="false">A</p><p pl-else>B</p>"#.into(), &vars);
    assert_eq!(result.unwrap(), "<p>B</p>");
}

#[test]
fn caret_attr_eval() {
    let vars = Map::new().into();

    let result = render(r#"<input ^value='"my" + " " + "name"'>"#.into(), &vars);
    assert_eq!(result.unwrap(), "<input value='my name'>");
}

#[test]
fn correct_escaping() {
    let vars = json!({"x": "<code>&lt;TAG&gt;</code>"});

    let result = render(r#"<slot pl-html="x"></slot>"#.into(), &vars);
    assert_eq!(result.unwrap(), "<slot><code>&lt;TAG&gt;</code></slot>");
}

#[test]
fn caret_attr_false() {
    let vars = Map::new().into();

    let result = render(r#"<input ^disabled='false'>"#.into(), &vars);
    assert_eq!(result.unwrap(), "<input>");
}

#[test]
fn caret_attr_array() {
    let vars = Map::new().into();

    let result = render(
        r#"<button ^class='["warn", "error"]'></button>"#.into(),
        &vars,
    );
    assert_eq!(result.unwrap(), "<button class='warn error'></button>");
}

#[test]
fn caret_attr_object() {
    let vars = json!({ "classes": { "should-have": true, "should-not-have": null, "should-also-have": 1 } });

    let result = render(r#"<button ^class='classes'></button>"#.into(), &vars);
    assert_eq!(
        result.unwrap(),
        "<button class='should-have should-also-have'></button>"
    );
}

#[test]
fn comments_uneffected() {
    let vars = Map::new().into();

    let result = render(r#"<!-- MAKE ART NOT SOFTWARE -->"#.into(), &vars);
    assert_eq!(result.unwrap(), "<!-- MAKE ART NOT SOFTWARE -->");
}

#[test]
fn order_unchanged() {
    let vars = Map::new().into();

    let result = render(
        r#"<meta ^disabled="false" name="x" ^content='"y"'>"#.into(),
        &vars,
    );
    assert_eq!(result.unwrap(), "<meta name='x' content='y'>");
}

#[test]
fn for_loop_array_index() {
    let vars = Map::new().into();

    let result = render(
        r#"<hr pl-for="(x, i) in [1,2,3]" ^name="x" ^class="i">"#.into(),
        &vars,
    );
    assert_eq!(
        result.unwrap(),
        "<hr name='1' class='0'><hr name='2' class='1'><hr name='3' class='2'>"
    );
}

#[test]
fn for_loop_kv() {
    let vars = json!({"fields": {"first-name": "First Name", "last-name": "Last Name"}});

    let result = render(
        r#"<input pl-for="(k, v) in fields" ^name="k" ^placeholder="v">"#.into(),
        &vars,
    );
    assert_eq!(
        result.unwrap(),
        "<input name='first-name' placeholder='First Name'>\
             <input name='last-name' placeholder='Last Name'>"
    );
}

#[test]
fn for_loop_kvi() {
    let vars = json!({"fields": {"first-name": "First Name", "last-name": "Last Name"}});

    let result = render(
        r#"<input pl-for="(k, v, i) in fields" ^name="k + '-' + i" ^placeholder="v">"#.into(),
        &vars,
    );
    assert_eq!(
        result.unwrap(),
        "<input name='first-name-0' placeholder='First Name'>\
             <input name='last-name-1' placeholder='Last Name'>"
    );
}

#[test]
fn for_loop_if_else_if() {
    let vars = Map::new().into();

    let result = render(
        "<div pl-if='false'>A</div>\
                      <div pl-for='x in [1,2,3]' pl-else-if='true'>B</div>\
                      <div>C</div>"
            .into(),
        &vars,
    );
    assert_eq!(
        result.unwrap(),
        "<div>B</div><div>B</div><div>B</div><div>C</div>"
    );
}

#[test]
fn bad_pl_is_name() {
    let vars = Map::new().into();

    let result = render("<div pl-is='\"\"'></div>".into(), &vars);
    assert_eq!(
        result.unwrap_err(),
        RenderError {
            filename: "input".to_owned(),
            kind: RenderErrorKind::BadPlIsName("".to_string())
        }
    );
}

#[test]
fn pl_for_and_pl_is() {
    let vars = Map::new().into();

    let result = render(
        r#"<div pl-for='x in ["h1", "h2", "h3"]' pl-is='x'></div>"#.into(),
        &vars,
    );
    assert_eq!(result.unwrap(), "<h1></h1><h2></h2><h3></h3>");
}

#[test]
fn only_include_styles_once() {
    let vars = Map::new().into();

    let result = render(
        r#"<style> * { color: red; } </style><style> * { color: red; } </style>"#.into(),
        &vars,
    );
    assert_eq!(result.unwrap(), "<style> * { color: red; } </style>");
}

#[test]
fn include_styles_or_scripts_twice_different_args() {
    let vars = Map::new().into();

    let result = render(
        r#"<style attr> * { color: red; } </style><style> * { color: red; } </style>"#.into(),
        &vars,
    );
    assert_eq!(
        result.unwrap(),
        "<style attr=''> * { color: red; } </style><style> * { color: red; } </style>"
    );
}

#[test]
fn no_script_injection() {
    let vars = json!({"username": "'); alert('you have been hacked'); let x = ('"});

    let result = render(
        r#"<script> console.log('{{username}}') </script>"#.into(),
        &vars,
    );
    assert_eq!(
        result.unwrap(),
        "<script> console.log('{{username}}') </script>"
    );
}

#[test]
fn no_html_text_injection() {
    let vars =
        json!({ "username": "username</div><script>alert('you have been hacked');</script>'" });

    let result = render(r#"<div>{{username}}</div>"#.into(), &vars);
    assert_eq!(
        result.unwrap(),
        "<div>username&lt;/div&gt;&lt;script&gt;alert('you have been hacked');&lt;/script&gt;'</div>"
    );
}

#[test]
fn pl_else_if_template() {
    let vars = Map::new().into();

    let result = render(
        r#"<div pl-if='0'>hello</div><template pl-else-if='1'>this</template>"#.into(),
        &vars,
    );
    assert_eq!(result.unwrap(), "this");
}

#[test]
fn pl_else_template() {
    let vars = Map::new().into();

    let result = render(
        r#"<div pl-if='0'>hello</div><template pl-else>this</template>"#.into(),
        &vars,
    );
    assert_eq!(result.unwrap(), "this");
}

#[test]
fn pl_is_and_pl_html() {
    let vars = Map::new().into();

    let result = render(
        r#"<div pl-is='"dialog"' pl-html='"hello!"'></div>"#.into(),
        &vars,
    );

    assert_eq!(result.unwrap(), "<dialog>hello!</dialog>");
}

#[test]
fn pl_template_and_pl_if() {
    let vars = Map::new().into();

    let result = render(r#"<template pl-if=true>{{ 99 }}</template>"#.into(), &vars);

    assert_eq!(result.unwrap(), "99");
}
