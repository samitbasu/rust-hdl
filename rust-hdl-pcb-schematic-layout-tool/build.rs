// Copyright 2020 The Druid Authors.
//
// Licensed under the Apache License, Version 2.0 (the "License");
// you may not use this file except in compliance with the License.
// You may obtain a copy of the License at
//
//     http://www.apache.org/licenses/LICENSE-2.0
//
// Unless required by applicable law or agreed to in writing, software
// distributed under the License is distributed on an "AS IS" BASIS,
// WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
// See the License for the specific language governing permissions and
// limitations under the License.

use std::io::{ErrorKind, Result};
use std::path::{Path, PathBuf};
use std::{env, fs};

fn main() -> Result<()> {
    let crate_dir = PathBuf::from(&env::var("CARGO_MANIFEST_DIR").unwrap());
    let src_dir = crate_dir.join("src");
    let examples_dir = src_dir.join("examples");

    let parent_dir = crate_dir.parent().unwrap();

    // Generate example module and the necessary html documents.

    let mut index_html = r#"<!DOCTYPE html>
<html lang="en">
    <head>
        <meta charset="utf-8">
        <title>Druid web examples</title>
    </head>
    <body>
        <h1>Druid web examples</h1>
        <ul>
"#
    .to_string();

    // if let Some(example) = path.file_stem() {
    let example_str = "rust_hdl_pcb_schematic_layout_tool";

    // Add an entry to the index.html file.
    let index_entry = format!(
        "<li><a href=\"./html/{name}.html\">{name}</a></li>",
        name = example_str
    );

    index_html.push_str(&index_entry);

    // Create an html document for each example.
    let html = format!(
        r#"
<!DOCTYPE html>
<html lang="en">
    <head>
        <meta charset="utf-8">
        <title>Druid web examples - {name}</title>
        <style>
            html, body, canvas {{
                margin: 0px;
                padding: 0px;
                width: 100%;
                height: 100%;
                overflow: hidden;
            }}
        </style>
    </head>
    <body>
        <noscript>This page contains WebAssembly and JavaScript content, please enable JavaScript in your browser.</noscript>
        <canvas id="canvas"></canvas>
        <script type="module">
            import init, {{ {name} }} from '../pkg/rust_hdl_pcb_schematic_layout_tool.js';
            async function run() {{
                await init();
                {name}();
            }}
            run();
        </script>
    </body>
</html>"#,
        name = example_str.to_string()
    );

    // Write out the html file into a designated html directory located in crate root.
    let html_dir = crate_dir.join("html");
    if !html_dir.exists() {
        fs::create_dir(&html_dir)
            .unwrap_or_else(|_| panic!("Failed to create output html directory: {:?}", &html_dir));
    }

    fs::write(html_dir.join(example_str).with_extension("html"), html)
        .unwrap_or_else(|_| panic!("Failed to create {}.html", example_str));
    // }
    // }

    index_html.push_str("</ul></body></html>");

    // Write out the index.html file
    fs::write(crate_dir.join("index.html"), index_html)?;

    Ok(())
}
