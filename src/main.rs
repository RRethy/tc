use tree_sitter::{Language, Parser};

extern "C" {
    fn tree_sitter_rust() -> Language;
}

fn main() {
    let mut parser = Parser::new();
    let language = unsafe { tree_sitter_rust() };
    parser.set_language(language).unwrap();
    let source_code = "fn test() { let mut foo = \"hello\";}";
    let tree = parser.parse(source_code, None).unwrap();
    let root_node = tree.root_node();
    for x in root_node.children(&mut root_node.walk()) {
        if let Ok(name) = x.utf8_text(source_code.as_bytes()) {
            println!("{}", name);
        } else {
        }
        println!("{}", x.to_sexp());
        for y in x.children(&mut x.walk()) {
            if let Ok(name) = y.utf8_text(source_code.as_bytes()) {
                println!("{}", name);
            } else {
            }
            println!("	{}", y.to_sexp());
            for z in y.children(&mut y.walk()) {
                if let Ok(name) = z.utf8_text(source_code.as_bytes()) {
                    println!("{}", name);
                } else {
                }
                println!("		{}", z.to_sexp());
                for a in z.children(&mut z.walk()) {
                    if let Ok(name) = a.utf8_text(source_code.as_bytes()) {
                        println!("{}", name);
                    } else {
                    }
                    println!("			{}", a.to_sexp());
                    for b in a.children(&mut a.walk()) {
                        if let Ok(name) = b.utf8_text(source_code.as_bytes()) {
                            println!("{}", name);
                        } else {
                        }
                        println!("				{}", b.to_sexp());
                        for c in b.children(&mut b.walk()) {
                            if let Ok(name) = c.utf8_text(source_code.as_bytes()) {
                                println!("{}", name);
                            } else {
                            }
                            println!("					{}", c.to_sexp());
                        }
                    }
                }
            }
        }
    }
}
