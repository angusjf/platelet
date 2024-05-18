use std::collections::HashMap;

use platelet::{render_to_string, renderer::Filesystem};
use serde_json::{json, Map};

struct MockMultiFile {
    data: HashMap<String, String>,
}

impl Filesystem for MockMultiFile {
    fn read(&self, path: &String) -> String {
        self.data.get(path).unwrap().clone()
    }
    fn move_to(&self, _current: &String, path: &String) -> String {
        path.to_owned()
    }
}

#[test]
fn pl_src() {
    let vars = Map::new().into();

    let result = render_to_string(
        &vars,
        &"index.html".into(),
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

    let result = render_to_string(
            &vars,
            &"index.html".into(),
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

    let result = render_to_string(
        &vars,
        &"index.html".into(),
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

    let result = render_to_string(
        &vars,
        &"index.html".into(),
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

    let result = render_to_string(
        &vars,
        &"index.html".into(),
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

    let result = render_to_string(
        &vars,
        &"index.html".into(),
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
              <style>article {display: flex;}</style>\
           </body>\
         </html>"
    );
}
