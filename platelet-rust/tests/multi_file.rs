use std::collections::HashMap;

use platelet::{render_with_custom_filesystem, renderer::Filesystem};
use serde_json::{json, Map};

struct MockMultiFile {
    data: HashMap<String, String>,
}

impl Filesystem<()> for MockMultiFile {
    fn read(&self, path: &String) -> Result<String, ()> {
        Ok(self.data.get(path).unwrap().clone())
    }
    fn move_to(&self, _current: &String, path: &String) -> Result<String, ()> {
        Ok(path.to_owned())
    }
}

#[test]
fn pl_src() {
    let vars = Map::new().into();

    let result = render_with_custom_filesystem(
        &"index.html".into(),
        &vars,
        &MockMultiFile {
            data: HashMap::from([
                (
                    "index.html".into(),
                    "<article><slot pl-src='embed.html'></slot></article>".to_owned(),
                ),
                ("embed.html".into(), "<p>hello world</p>".to_owned()),
            ]),
        },
    );
    assert_eq!(result.unwrap(), "<article><p>hello world</p></article>");
}

#[test]
fn pl_src_with_slot() {
    let vars = Map::new().into();

    let result = render_with_custom_filesystem(
            &"index.html".into(),
            &vars,
            &MockMultiFile {
                data: HashMap::from([
                    (
                        "index.html".into(),
                        "<article><slot pl-src='embed.html'><b>inner</b><b>content</b></slot></article>".to_owned(),
                    ),
                    ("embed.html".into(), "<div><slot pl-slot></slot></div>".to_owned()),
                ]),
            },
        );
    assert_eq!(
        result.unwrap(),
        "<article><div><b>inner</b><b>content</b></div></article>"
    );
}

#[test]
fn pl_src_with_named_slots() {
    let vars = Map::new().into();

    let result = render_with_custom_filesystem(
        &"index.html".into(),
        &vars,
        &MockMultiFile {
            data: HashMap::from([
                (
                    "index.html".into(),
                    "<slot pl-src='embed.html'>\
                             <template pl-slot='left'><b>Left</b> hand side</template>\
                             <template pl-slot='right'><b>Right</b> hand side</template>\
                         </slot>"
                        .to_owned(),
                ),
                (
                    "embed.html".into(),
                    "<left><slot pl-slot='left'></slot></left>\
                         <right><slot pl-slot='right'></slot></right>"
                        .to_owned(),
                ),
            ]),
        },
    );
    assert_eq!(
        result.unwrap(),
        "<left><b>Left</b> hand side</left><right><b>Right</b> hand side</right>"
    );
}

#[test]
fn pl_src_with_cotext() {
    let vars = Map::new().into();

    let result = render_with_custom_filesystem(
        &"index.html".into(),
        &vars,
        &MockMultiFile {
            data: HashMap::from([
                (
                    "index.html".into(),
                    r#"<slot pl-src='embed.html' ^message='"hello world"'></slot>"#.to_owned(),
                ),
                ("embed.html".into(), "<code>{{message}}</code>".to_owned()),
            ]),
        },
    );
    assert_eq!(result.unwrap(), "<code>hello world</code>");
}

#[test]
fn pl_for_and_pl_src() {
    let vars = Map::new().into();

    let result = render_with_custom_filesystem(
        &"index.html".into(),
        &vars,
        &MockMultiFile {
            data: HashMap::from([
                (
                    "index.html".into(),
                    "<slot pl-for='a in [1,2,3]' pl-src='x.html' ^x='a'></slot>".to_owned(),
                ),
                ("x.html".into(), "<h1>{{x}}</h1>".to_owned()),
            ]),
        },
    );
    assert_eq!(result.unwrap(), "<h1>1</h1><h1>2</h1><h1>3</h1>");
}

#[test]
fn example() {
    let vars = json!({
      "title": "Angus' Blog",
      "blogposts": [
        {
          "img_url": "http://angusjf.com/x.png",
          "link": "http://angusjf.com/",
          "summary": "<p>hello world</p>",
          "title": "SOMETHING COOL",
          "date": "01/11/2025"
        },
        {
          "img_url": "http://angusjf.com/x.png",
          "link": "http://angusjf.com/",
          "summary": "<p>hello world</p>",
          "title": "SOMETHING ELSE",
          "date": "01/11/2025"
        }
      ]
    });

    let result = render_with_custom_filesystem(
        &"index.html".into(),
        &vars,
        &MockMultiFile {
            data: HashMap::from([
                (
                    "index.html".into(),
                    "<!doctype html>\
                     <html>\
                       <head>\
                         <title>{{ title }}</title>\
                       </head>\
                       <body>\
                         <slot pl-for='b in blogposts' pl-src='blogpost.html' ^blogpost='b'>\
                         </slot>\
                       </body>\
                      </html>"
                        .to_owned(),
                ),
                (
                    "blogpost.html".into(),
                    "<article>\
                        <img ^src='blogpost.img_url'>\
                        <div>\
                            <h2>\
                                <a ^href='blogpost.link'>{{blogpost.title}}</a>\
                            </h2>\
                            <template pl-html='blogpost.summary'></template>\
                            <date>{{blogpost.date}}</date>\
                        </div>\
                    </article>\
                    <style>\
                        article {\
                            display: flex;\
                        }\
                    </style>\
                    "
                    .to_owned(),
                ),
            ]),
        },
    );
    assert_eq!(
        result.unwrap(),
        "<!DOCTYPE html>\
         <html>\
          <head>\
            <title>Angus' Blog</title>\
          </head>\
          <body>\
            <article>\
              <img src='http://angusjf.com/x.png'>\
              <div>\
                <h2><a href='http://angusjf.com/'>SOMETHING COOL</a></h2>\
                <p>hello world</p>\
                <date>01/11/2025</date>\
              </div>\
             </article>\
             <style>article {display: flex;}</style>\
             <article>\
               <img src='http://angusjf.com/x.png'>\
               <div>\
                 <h2><a href='http://angusjf.com/'>SOMETHING ELSE</a></h2>\
                 <p>hello world</p>\
                 <date>01/11/2025</date>\
               </div>\
              </article>\
           </body>\
         </html>"
    );
}

#[test]
fn pl_for_lots_of_children() {
    let vars = Map::new().into();

    let result = render_with_custom_filesystem(
        &"index.html".into(),
        &vars,
        &MockMultiFile {
            data: HashMap::from([
                (
                    "index.html".into(),
                    r#"<ol><slot ^i=i ^s=s pl-for='(s, i) in ["my", "cousin", "carl", "duckworth", "said"]' pl-src="embed.html"></slot></ol>"#.to_owned(),
                ),
                ("embed.html".into(), "<div>{{i}}</div>({{s}})<script>console.log()</script>".to_owned()),
            ]),
        },
    );
    assert_eq!(
        result.unwrap(),
        "<ol><div>0</div>(my)\
        <script>console.log()</script>\
        <div>1</div>(cousin)\
        <div>2</div>(carl)\
        <div>3</div>(duckworth)\
        <div>4</div>(said)</ol>"
    );
}

#[test]
#[ignore]
fn nested_slots() {
    let vars = json!({});

    let result = render_with_custom_filesystem(
        &"index.html".into(),
        &vars,
        &MockMultiFile {
            data: HashMap::from([
                (
                    "index.html".into(),
                    r#"<!DOCTYPE html>
                    <html lang='en'>
                        <head><title>hello world</title></head>
                        <body>
                            <slot pl-src='navbar.html'>
                              <template pl-slot='dropdown'></template>
                              <template pl-slot='button'></template>
                            </slot>

                            <div class='section'>
                                <div class='title'>Login</div>
                                <form action='/login' method='post'>
                                    <slot pl-src='input.html'></slot>
                                    <slot pl-src='input.html'></slot>
                                    <slot pl-src='button.html'></slot>
                                </form>
                            </div>
                        </body>

                    </html>"#
                        .to_owned(),
                ),
                (
                    "head.html".into(),
                    "<head><title>hello world</title></head>".to_owned(),
                ),
                (
                    "navbar.html".into(),
                    r#"<div class='navbar'>
                        <a class='logo' href='/'>My App</a>

                        <div class='actions'>
                            <slot pl-slot='dropdown'></slot>
                            <slot pl-slot='button'></slot>
                        </div>
                    </div>"#
                        .to_owned(),
                ),
                ("input.html".into(), r#"<input>"#.to_owned()),
                (
                    "button.html".into(),
                    r#"<button class='button-container'>
                        <a
                            ^class='["button", is_primary && "primary"]'
                            ^type='is_submit && "submit"'
                            ^href='link'
                        >
                            {{text}}
                        </a>
                    </button>"#
                        .to_owned(),
                ),
            ]),
        },
    );
    assert_eq!(
        result.unwrap(),
        r#"<!DOCTYPE html><html lang='en'>
        <head><title>hello world</title></head><body></body></html>\n                        </head><body>\n                            <div class='navbar'>\n                        <a class='logo' href='/'>My App</a>\n\n                        <div class='actions'>\n                            \n                            \n                        </div>\n                    </div>\n\n                            <div class='section'>\n                                <div class='title'>Login</div>\n                                <form action='/login' method='post'>\n                                    {template 'input' .EmailInput}}\n                                    {template 'input' .PasswordInput}}\n                                    {template 'button' .SubmitButton}}\n                                </form>\n                            </div>\n                        \n\n                    </body></html>

        "#
    );
}

#[test]
fn nested_slots_min() {
    let vars = json!({});

    let result = render_with_custom_filesystem(
        &"index.html".into(),
        &vars,
        &MockMultiFile {
            data: HashMap::from([
                (
                    "index.html".into(),
                    "<div pl-src='navbar.html'>
                       <template pl-slot='dropdown'>
                         x
                       </template>
                       <template pl-slot='button'>
                         y
                       </template>
                     </div>"
                        .to_owned(),
                ),
                (
                    "navbar.html".into(),
                    "<span class='navbar'>
                       <slot pl-slot='dropdown'></slot>
                       <slot pl-slot='button'></slot>
                     </span>"
                        .to_owned(),
                ),
            ]),
        },
    );
    assert_eq!(
        result.unwrap(),
        format!(
            "{}{}{}{}{}{}{}{}",
            "<span class='navbar'>\n",
            "                       \n",
            "                         x\n",
            "                       \n",
            "                       \n",
            "                         y\n",
            "                       \n",
            "                     </span>"
        )
    );
}
