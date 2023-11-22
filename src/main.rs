mod kod;
use std::path;

use kod::{lexer::lexer::Lexer, parser::parser::Parser, compiler::bytekod::{Code, Module}, runtime::runtime::VM};

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
    let tree = parser.parse().unwrap();
    std::fs::write(path::Path::new("output.txt"), &tree.to_string()).expect("Unable to write file");
    print!("{}", tree.to_string());

    let mut module = Module { name: filename.to_string(), name_pool: vec![], constant_pool: vec![], entry: Code::new("__main__".to_string(), vec![], vec![]) };
    let mut entry = Code::new("__main__".to_string(), vec![], vec![]);
    tree.compile(&mut module, &mut entry);

    module.entry = entry;

    module.entry.print();
    println!("{:#?}", module.constant_pool);
    println!("{:#?}", module.name_pool);

    let mut vm = VM::new(module);
    vm.run(&tree);

    Ok(())
}
