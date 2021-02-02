use reqwest::blocking as rq;
use std::env;
use std::error::Error;
use std::fs::{self, File};
use std::io::{prelude::*, BufReader};
use std::iter;

const INPUT_FILENAME: &str = "Joos-1W-features";
const FEATURES_URL: &str = "https://student.cs.uwaterloo.ca/~cs444/features";
const OUTPUT_DIR: &str = "features";

type Res<T> = Result<T, Box<dyn Error>>;

fn main() -> Res<()> {
    let input = BufReader::new(File::open(INPUT_FILENAME)?);

    fs::remove_dir_all(OUTPUT_DIR)?;
    fs::create_dir(OUTPUT_DIR)?;
    env::set_current_dir(OUTPUT_DIR)?;

    let mut dir = None;
    for line in input.lines() {
        let line = line?;

        if line.chars().all(|c| c.is_whitespace()) {
            // Skip blank lines.
        } else if line.starts_with("#") {
            // `mkdir` for this group of features.
            let name = to_filename(&line[1..]);
            fs::create_dir(&name)?;
            dir = Some(name);
        } else {
            // Download the example; read it into memory.
            let name = to_filename(&line);
            let url = String::from(FEATURES_URL) + "/" + &remove_dashes(&name) + ".html";
            let res = rq::get(&url)?;
            assert!(res.status().is_success(), url);
            let lines: Vec<_> = BufReader::new(res).lines().collect::<Result<_, _>>()?;

            // Write it to a file, stripping out certain HTML tags.
            let filename = String::from(dir.as_ref().unwrap()) + "/" + &name + ".java";
            let mut file = File::create(&filename)?;
            for line in strip_tags(lines.iter().map(|s| s.as_str())) {
                writeln!(file, "{}", line)?;
            }

            println!("Created {}", filename);
        }
    }

    Ok(())
}

/// Trim whitespace; make lowercase; replace spaces with dashes.
///
/// `line` should consist of ascii-alphabetic, spaces, underscores.
fn to_filename(line: &str) -> String {
    assert!(line
        .chars()
        .all(|c| " -_".contains(c) || c.is_ascii_alphabetic()));

    let line = line.trim();
    let line = line.to_lowercase();
    line.chars()
        .map(|c| if c == ' ' { '-' } else { c })
        .collect()
}

fn remove_dashes(name: &str) -> String {
    name.replace('-', "")
}

/// Strip out some very specific HTML formatting from the body.
///
/// Also resolves these escapes: &amp; &lt; &gt;
fn strip_tags<'a>(
    mut lines: impl Iterator<Item = &'a str> + 'a,
) -> impl Iterator<Item = String> + 'a {
    iter::from_fn(move || loop {
        match lines.next() {
            None => break None,
            Some("<pre>") | Some("</pre>") => continue,
            Some(line) => {
                let mut line = line.replace("<font color=\"blue\">", "");
                line = line.replace("<font color=blue>", "");
                line = line.replace("</font>", "");

                line = line.replace("&amp;", "&");
                line = line.replace("&lt;", "<");
                line = line.replace("&gt;", ">");

                break Some(line);
            }
        }
    })
}
