use platelet::{render_string_to_string, renderer::RenderError};
use serde_json::{json, Map};

#[test]
fn templateless_text_node() {
    let vars = json!({ "hello": "world" });

    let result = render_string_to_string(&vars, "<h1>nothing here</h1>".into());
    assert_eq!(result.unwrap(), "<h1>nothing here</h1>");
}

#[test]
fn templateless_html_doc() {
    let vars = json!({ "hello": "world" });

    let result = render_string_to_string(
        &vars,
        "<!doctype html><html><head><title>a</title></head><body></body></html>".into(),
    );
    assert_eq!(
        result.unwrap(),
        "<!DOCTYPE html><html><head><title>a</title></head><body></body></html>"
    );
}

#[test]
fn templated_text_node() {
    let vars = json!({ "hello": "world" });

    let result = render_string_to_string(&vars, "<h1>{{hello}}</h1>".into());
    assert_eq!(result.unwrap(), "<h1>world</h1>");
}

#[test]
fn complex_text_node() {
    let vars = json!({ "user": {"firstname": "Yuri", "lastname" : "Gagarin" } });

    let result = render_string_to_string(
        &vars,
        "<h1>Dear {{user.firstname}} {{user.lastname}},</h1>".into(),
    );
    assert_eq!(result.unwrap(), "<h1>Dear Yuri Gagarin,</h1>");
}

#[test]
fn text_node_with_expressions() {
    let vars = json!({ "countries": [ "portugal" ] });

    let result = render_string_to_string(&vars, "<h1>{{countries[0]}} {{ 1 + 2 }}</h1>".into());
    assert_eq!(result.unwrap(), "<h1>portugal 3</h1>");
}

#[test]
fn pl_if() {
    let vars = Map::new().into();

    let result = render_string_to_string(
        &vars,
        "<p>this</p>\
                    <p pl-if='false'>not this</p>\
                    <p>also this</p>"
            .into(),
    );
    assert_eq!(result.unwrap(), "<p>this</p><p>also this</p>");
}

#[test]
fn pl_else_if_true() {
    let vars = Map::new().into();

    let result = render_string_to_string(
        &vars,
        "<p>this</p>\
                        <p pl-if='false'>not this</p>\
                        <p pl-else-if='true'>also this</p>"
            .into(),
    );
    assert_eq!(result.unwrap(), "<p>this</p><p>also this</p>");
}

#[test]
fn pl_else_if_false() {
    let vars = Map::new().into();

    let result = render_string_to_string(
        &vars,
        "<p>this</p>\
                        <p pl-if='false'>not this</p>\
                        <p pl-else-if='false'>also this</p>"
            .into(),
    );
    assert_eq!(result.unwrap(), "<p>this</p>");
}

#[test]
fn pl_is() {
    let vars = json!({ "header": true });

    let result =
        render_string_to_string(&vars, "<p pl-is='header ? \"h1\" : \"h2\"'>this</p>".into());
    assert_eq!(result.unwrap(), "<h1>this</h1>");
}

#[test]
fn pl_html() {
    let vars = json!({ "content": "<p>hello world</p>" });

    let result = render_string_to_string(
        &vars,
        "<article pl-html='content'>something that used to be here</article>".into(),
    );
    assert_eq!(result.unwrap(), "<article><p>hello world</p></article>");
}

#[test]
fn pl_html_with_vars_are_not_rendered() {
    let vars = json!({ "content": "<p>hello {{mistake}} world</p>" });

    let result = render_string_to_string(
        &vars,
        "<article pl-html='content'>something that used to be here</article>".into(),
    );
    assert_eq!(
        result.unwrap(),
        "<article><p>hello {{mistake}} world</p></article>"
    );
}

#[test]
fn pl_html_with_template() {
    let vars = json!({ "content": "<p>hello world</p>" });

    let result = render_string_to_string(
        &vars,
        "<template pl-html='content'>something that used to be here</template>".into(),
    );
    assert_eq!(result.unwrap(), "<p>hello world</p>");
}

#[test]
fn template_preserved() {
    let vars = Map::new().into();

    let result = render_string_to_string(&vars, "<template><h1>hello</h1></template>".into());
    assert_eq!(result.unwrap(), "<template><h1>hello</h1></template>");
}

#[test]
fn pl_for() {
    let vars = Map::new().into();

    let result = render_string_to_string(
        &vars,
        "<div><p pl-for='x in [1,2,3]'>{{x}}</p></div>".into(),
    );
    assert_eq!(result.unwrap(), "<div><p>1</p><p>2</p><p>3</p></div>");
}

#[test]
fn pl_for_template() {
    let vars = Map::new().into();

    let result = render_string_to_string(
        &vars,
        "<div><template pl-for='x in [1,2,3]'><p>{{x}}</p></template></div>".into(),
    );
    assert_eq!(result.unwrap(), "<div><p>1</p><p>2</p><p>3</p></div>");
}

#[test]
fn pl_if_template() {
    let vars = Map::new().into();

    let result = render_string_to_string(
        &vars,
        "<div><template pl-if='[1]'><p>hello</p><p>world</p></template></div>".into(),
    );
    assert_eq!(result.unwrap(), "<div><p>hello</p><p>world</p></div>");
}

#[test]
fn loop_with_if_else() {
    let vars = Map::new().into();

    let result = render_string_to_string(
        &vars,
        "<div pl-if='\"A\" == \"Z\"'>A</div>\
                         <div pl-for='_ in [1,3]' pl-else-if='\"A\" == \"A\"'>B</div>\
                         <div pl-else-if='\"A\" == \"A\"'>C</div>\
                         <div pl-else>Not A/B/C</div>"
            .into(),
    );
    assert_eq!(result.unwrap(), "<div>B</div><div>B</div>");
}

#[test]
fn pl_else_true() {
    let vars = Map::new().into();

    let result = render_string_to_string(&vars, r#"<p pl-if="true">A</p><p pl-else>B</p>"#.into());
    assert_eq!(result.unwrap(), "<p>A</p>");
}

#[test]
fn pl_else_false() {
    let vars = Map::new().into();

    let result = render_string_to_string(&vars, r#"<p pl-if="false">A</p><p pl-else>B</p>"#.into());
    assert_eq!(result.unwrap(), "<p>B</p>");
}

#[test]
fn caret_attr_eval() {
    let vars = Map::new().into();

    let result = render_string_to_string(&vars, r#"<input ^value='"my" + " " + "name"'>"#.into());
    assert_eq!(result.unwrap(), "<input value='my name'>");
}

#[test]
fn correct_escaping() {
    let vars = json!({"x": "<code>&lt;TAG&gt;</code>"});

    let result = render_string_to_string(&vars, r#"<slot pl-html="x"></slot>"#.into());
    assert_eq!(result.unwrap(), "<slot><code>&lt;TAG&gt;</code></slot>");
}

#[test]
fn caret_attr_false() {
    let vars = Map::new().into();

    let result = render_string_to_string(&vars, r#"<input ^disabled='false'>"#.into());
    assert_eq!(result.unwrap(), "<input>");
}

#[test]
fn caret_attr_array() {
    let vars = Map::new().into();

    let result = render_string_to_string(
        &vars,
        r#"<button ^class='["warn", "error"]'></button>"#.into(),
    );
    assert_eq!(result.unwrap(), "<button class='warn error'></button>");
}

#[test]
fn caret_attr_object() {
    let vars = json!({ "classes": { "should-have": true, "should-not-have": null, "should-also-have": 1 } });

    let result = render_string_to_string(&vars, r#"<button ^class='classes'></button>"#.into());
    assert_eq!(
        result.unwrap(),
        "<button class='should-also-have should-have'></button>"
    );
}

#[test]
fn comments_uneffected() {
    let vars = Map::new().into();

    let result = render_string_to_string(&vars, r#"<!-- MAKE ART NOT SOFTWARE -->"#.into());
    assert_eq!(result.unwrap(), "<!-- MAKE ART NOT SOFTWARE -->");
}

#[test]
fn order_unchanged() {
    let vars = Map::new().into();

    let result = render_string_to_string(
        &vars,
        r#"<meta ^disabled="false" name="x" ^content='"y"'>"#.into(),
    );
    assert_eq!(result.unwrap(), "<meta name='x' content='y'>");
}

#[test]
fn for_loop_array_index() {
    let vars = Map::new().into();

    let result = render_string_to_string(
        &vars,
        r#"<hr pl-for="(x, i) in [1,2,3]" ^name="x" ^class="i">"#.into(),
    );
    assert_eq!(
        result.unwrap(),
        "<hr name='1' class='0'><hr name='2' class='1'><hr name='3' class='2'>"
    );
}

#[test]
fn for_loop_kv() {
    let vars = json!({"fields": {"first-name": "First Name", "last-name": "Last Name"}});

    let result = render_string_to_string(
        &vars,
        r#"<input pl-for="(k, v) in fields" ^name="k" ^placeholder="v">"#.into(),
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

    let result = render_string_to_string(
        &vars,
        r#"<input pl-for="(k, v, i) in fields" ^name="k + '-' + i" ^placeholder="v">"#.into(),
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

    let result = render_string_to_string(
        &vars,
        "<div pl-if='false'>A</div>\
                      <div pl-for='x in [1,2,3]' pl-else-if='true'>B</div>\
                      <div>C</div>"
            .into(),
    );
    assert_eq!(
        result.unwrap(),
        "<div>B</div><div>B</div><div>B</div><div>C</div>"
    );
}

#[test]
fn bad_pl_is_name() {
    let vars = Map::new().into();

    let result = render_string_to_string(&vars, "<div pl-is='\"\"'></div>".into());
    assert_eq!(
        result.unwrap_err(),
        RenderError::BadPlIsName("".to_string())
    );
}

#[test]
fn pl_for_and_pl_is() {
    let vars = Map::new().into();

    let result = render_string_to_string(
        &vars,
        r#"<div pl-for='x in ["h1", "h2", "h3"]' pl-is='x'></div>"#.into(),
    );
    assert_eq!(result.unwrap(), "<h1></h1><h2></h2><h3></h3>");
}
