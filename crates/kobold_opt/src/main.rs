use std::fmt::{self, Debug};
use std::fs::File;
use std::io::prelude::*;
use std::io::BufReader;
use std::path::PathBuf;
use std::collections::HashMap;

use clap::Parser;

/// Simple program to greet a person
#[derive(Parser, Debug)]
#[command(version, about, long_about = None)]
struct Args {
    /// JavaScript file produced by wasm-bindgen
    input: PathBuf,

    /// Wasm file if different from `<input_without_extension>_bg.wasm`
    #[arg(short, long)]
    wasm: Option<PathBuf>,
}

struct Wasm<'source> {
    /// Maps import symbols to the index in `imports` vec.
    map: HashMap<&'source str, usize>,
    /// All imports
    imports: Vec<Import<'source>>,
    /// Wasm blob following imports
    tail: &'source[u8],
}

impl Debug for Wasm<'_> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("Wasm")
            .field("map", &self.map)
            .field("imports", &self.imports)
            .field("tail", &self.tail.len())
            .finish()
    }
}

impl<'source> Wasm<'source> {
    fn parse(source: &'source [u8]) -> Self {
        let mut i = 0;
        let mut last_stop = 0;

        let mut map = HashMap::new();
        let mut imports = Vec::new();

        while i < source.len() - 4 {
            if source[i..].starts_with(b"\x03wbg") {
                let len = source[i + 4] as usize;

                let slice = source
                    .get(i + 5..i + 5 + len)
                    .expect("Couldn't read import name from Wasm");

                let name = std::str::from_utf8(slice).expect("Invalid import name");

                map.insert(name, imports.len());

                imports.push(Import {
                    bytes: &source[last_stop..i],
                    name,
                });

                last_stop = i + 5 + len;
                i += 5 + 2 + len;
                continue;
            }

            i += 1;
        }

        Wasm {
            map,
            imports,
            tail: &source[last_stop..],
        }
    }
}

struct Import<'source> {
    // Unprased bytes before this particular input
    bytes: &'source [u8],
    // Found name
    name: &'source str,
}

impl Debug for Import<'_> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "<..{} bytes>{}", self.bytes.len(), self.name)
    }
}

fn main() -> std::io::Result<()> {
    let Args { input, wasm } = Args::parse();

    let wasm = wasm.unwrap_or_else(|| {
        let file = input
            .file_name()
            .expect("Missing file name")
            .to_str()
            .expect("Invalid characters in file name");
        let extension = input
            .extension()
            .expect("Missing file extension")
            .to_str()
            .expect("Invalid characters in file extension");

        if extension != "js" {
            panic!("Expected a `js` file extension on file: {file}");
        }

        input.with_file_name(format!("{}_bg.wasm", &file[..file.len() - 3]))
    });

    let wasm_bytes = {
        let mut file = BufReader::new(File::open(wasm)?);

        let mut contents = Vec::new();

        file.read_to_end(&mut contents)?;

        contents
    };

    let parsed = Wasm::parse(&wasm_bytes);

    println!(
        "Found {} imports amounting to {} bytes",
        parsed.imports.len(),
        parsed.imports.iter().map(|b| b.name.len()).sum::<usize>()
    );

    println!("{parsed:#?}");

    Ok(())
}
