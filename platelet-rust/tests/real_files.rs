use std::path::Path;

use platelet::render_with_filesystem;
use serde_json::json;

#[test]
fn pl_src() {
    let vars = json!({"examples": ["this", "that"]});

    let result = render_with_filesystem(&vars, Path::new("./tests/example_index.html"));

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