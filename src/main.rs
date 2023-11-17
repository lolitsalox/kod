mod kod;

use kod::{lexer::lexer::Lexer, parser::parser::Parser};

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

    print!("{}", parser.parse()?.to_string());
    Ok(())
}
