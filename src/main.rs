mod kod;
use std::{io::Write, path};

use kod::{lexer::lexer::Lexer, parser::parser::Parser, compiler::bytekod::{Code, Module}, runtime::runtime::VM};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let args: Vec<String> = std::env::args().collect();

    if args.len() < 2 {
        repl()
        // println!("Usage: {} <filename>", args[0]);
    }

    let filename = &args[1];
    let contents = std::fs::read_to_string(filename).unwrap();
    
    let lexer = Lexer::new(&filename, &contents);
    let mut parser = Parser::new(lexer);

    let tree = parser.parse().unwrap();

    // print!("{}", tree.to_string());
    // std::fs::write(path::Path::new("output.txt"), &tree.to_string()).expect("Unable to write file");
    
    let mut entry = Code::new("__main__".to_string(), vec![], vec![]);
    let mut module = Module { name: filename.to_string(), name_pool: vec![], constant_pool: vec![], entry: entry.clone() };
    
    tree.compile(&mut module, &mut entry);
    
    module.entry = entry;
    
    module.entry.print();
    println!("constant_pool: {:#?}", module.constant_pool);
    println!("name_pool: {:#?}", module.name_pool);

    let mut vm = VM::new(module);
    vm.run();

    Ok(())
}

fn repl() {
    
    let mut entry = Code::new("__main__".to_string(), vec![], vec![]);
    let mut module = Module { name: "repl".to_string(), name_pool: vec![], constant_pool: vec![], entry: entry.clone() };
    let mut vm = VM::new(module.clone());

    loop {
        print!(">>> ");
        std::io::stdout().flush().unwrap();
        let mut line = String::new();
        std::io::stdin().read_line(&mut line).unwrap();

        let lexer = Lexer::new("repl", &line);
        let mut parser = Parser::new(lexer);
        let tree = parser.parse().unwrap();
        
        tree.compile(&mut module, &mut entry);
        
        module.entry = entry.clone();
        vm.module = module.clone();

        module.entry.print();
        println!("constant_pool: {:#?}", module.constant_pool);
        println!("name_pool: {:#?}", module.name_pool);

        vm.run();
    }
}