mod kod;
use std::path;

use kod::{lexer::lexer::Lexer, parser::parser::Parser, compiler::compiler::JitCompiler};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    // let args: Vec<String> = std::env::args().collect();

    // if args.len() < 2 {
    //     println!("Usage: {} <filename>", args[0]);
    //     return Ok(());
    // }

    let filename = "script.kod";//&args[1];
    let contents = std::fs::read_to_string(filename).unwrap();
    
    let lexer = Lexer::new(&filename, &contents);
    let mut parser = Parser::new(lexer);
    // write the output of parser.parse()?.to_string() to a file
    let tree = parser.parse()?.to_string();
    std::fs::write(path::Path::new("output.txt"), &tree).expect("Unable to write file");
    print!("{tree}");

    let mut compiler = JitCompiler::new();

    compiler.compile(parser.parse()?);
    compiler.run();

    Ok(())
}
