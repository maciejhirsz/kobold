use std::fmt::{self, Debug};
use std::path::PathBuf;

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
    /// Byte size of the import section before transforms
    size: usize,
    /// Wasm blob before import section
    head: &'source [u8],
    /// All imports
    imports: Vec<Import<'source>>,
    /// Wasm blob following import section
    tail: &'source [u8],
}

impl Debug for Wasm<'_> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("Wasm")
            .field("head", &self.head)
            .field("imports", &self.imports)
            .field("tail", &self.tail.len())
            .finish()
    }
}

impl<'source> Wasm<'source> {
    fn parse(source: &'source [u8]) -> anyhow::Result<Self> {
        use wasmparser::{Parser, Payload};

        let parser = Parser::new(0);

        for payload in parser.parse_all(source) {
            let Payload::ImportSection(section) = payload? else {
                continue;
            };
            let mut head = &source[..section.range().start - 1];
            let size = section.range().end - section.range().start;
            let wasm = &source[section.range()];
            let tail = &source[section.range().end..];

            while let Some(last) = head.last() {
                // Stop if we are on the import section ID
                if *last == 2 {
                    break;
                }

                head = &head[..head.len() - 1];
            }

            let mut i = 0;
            let mut imports = Vec::with_capacity(section.count() as usize);

            while !wasm[i..].starts_with(b"\x03wbg") {
                i += 1;
            }

            while wasm[i..].starts_with(b"\x03wbg") {
                let len = wasm[i + 4] as usize;

                i += 5;

                let slice = wasm
                    .get(i..i + len)
                    .expect("Couldn't read import name from Wasm");

                let name = std::str::from_utf8(slice).expect("Invalid import name");

                i += len;

                imports.push(Import {
                    name,
                    ty: &wasm[i..i + 2],
                });

                i += 2;
                continue;
            }

            if imports.len() != section.count() as usize {
                panic!("Some imports not found?");
            }

            return Ok(Wasm {
                size,
                head,
                imports,
                tail,
            });
        }

        panic!("No import section in Wasm");
    }
}

struct Import<'source> {
    // Found name
    name: &'source str,
    // Unprased Wasm bytes for the type of this import
    ty: &'source [u8],
}

impl Debug for Import<'_> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{} -> {:?}", self.name, self.ty)
    }
}

fn main() -> anyhow::Result<()> {
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

    let wasm_bytes = std::fs::read(&wasm)?;

    let parsed = Wasm::parse(&wasm_bytes)?;

    // println!(
    //     "Found {} imports amounting to {} bytes",
    //     parsed.imports.len(),
    //     parsed.imports.iter().map(|b| b.name.len()).sum::<usize>()
    // );

    let js = std::fs::read_to_string(&input)?;

    let mut remaining = js.as_str();

    let mut sym = String::with_capacity(4);
    let mut js_new = String::with_capacity(js.len());
    let mut wasm_new = Vec::with_capacity(wasm_bytes.len());
    let mut wasm_imports = Vec::with_capacity(parsed.size);

    wasm_new.extend_from_slice(parsed.head);

    leb128::write::unsigned(&mut wasm_imports, parsed.imports.len() as u64)?;

    let mut saved = 0;

    for (n, import) in parsed.imports.into_iter().enumerate() {
        let Some(idx) = remaining.find(import.name) else {
            panic!(
                "Couldn't find the import {} in the JavaScript input",
                import.name
            );
        };

        symbol(n, &mut sym);

        // println!("Renaming {} to {sym}", import.name);

        saved += import.name.len() as isize - sym.len() as isize;

        js_new.push_str(&remaining[..idx]);
        js_new.push_str(&sym);
        wasm_imports.extend_from_slice(b"\x03wbg");
        wasm_imports.push(sym.len() as u8);
        wasm_imports.extend_from_slice(sym.as_bytes());
        wasm_imports.extend_from_slice(import.ty);

        remaining = &remaining[idx + import.name.len()..];
    }

    js_new.push_str(remaining);

    leb128::write::unsigned(&mut wasm_new, wasm_imports.len() as u64)?;

    wasm_new.extend_from_slice(&wasm_imports);
    wasm_new.extend_from_slice(parsed.tail);

    println!("Reduced both Wasm and JavaScript files by {saved} bytes");

    std::fs::write(&input, &js_new)?;
    std::fs::write(&wasm, &wasm_new)?;

    Ok(())
}

fn symbol(mut n: usize, buf: &mut String) {
    pub const ALPHABET: [u8; 62] = *b"abcdefghijklmnopqrstuvwxyz\
                                      ABCDEFGHIJKLMNOPQRSTUVWXYZ\
                                      0123456789";

    buf.clear();

    loop {
        let byte = ALPHABET[n % 62];

        buf.push(byte as char);

        n /= 62;

        if n == 0 {
            break;
        }
    }
}
