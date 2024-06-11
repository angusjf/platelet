use std::path::Path;

use platelet::render_file;
use serde_json::json;

#[test]
fn pl_src() {
    let vars = json!({"examples": ["this", "that"]});

    let result = render_file(Path::new("./tests/example_index.html"), &vars);

    assert_eq!(
        result.unwrap(),
        r#"<!DOCTYPE html><html><head><title>demo</title></head>
    <body>
        <table>
    <tbody><tr>
        <th>Name</th>
    </tr>
    <tr>
        <td>Maria Sanchez</td>
    </tr>
</tbody></table>
</body></html>"#
    );
}
