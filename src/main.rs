use {
    once_cell::sync::Lazy,
    regex::Regex,
};

use std::collections::HashMap;


fn lexer<'a>(input: &'a str) -> Vec<&'a str> {
    input.split_whitespace().collect()
}

#[derive(Debug, Clone, PartialEq, Eq)]
enum Node {
    Number(i64),
    Word(String),
    Function(fn(&mut Context) -> ()),
    Proc(Vec<Node>),
    StartDefinition,
    EndDefinition,
}

impl Node {
    fn get_number(&self) -> Option<i64> {
        match self {
            Node::Number(i) => Some(*i),
            _ => None,
        }
    }
    fn get_function(&self) -> Option<fn(&mut Context) -> ()> {
        match self {
            Node::Function(f) => Some(*f),
            _ => None,
        }
    }
    fn get_word(&self) -> Option<String> {
        match self {
            Node::Word(word) => Some(word.clone()),
            _ => None,
        }
    }
}

struct Context {
    stack: Vec<Node>,
    dict: HashMap<String, Node>,
}

impl Context {
    fn new() -> Context {
        Context {
            stack: Vec::new(),
            dict: HashMap::default(),
        }
    }

    fn register_function(&mut self, name: &str, f: fn(&mut Context) -> ()) {
        self.dict.insert(name.to_string(), Node::Function(f));
    }

}

fn parse_token(token: &str) -> Node {
    static INTEGER_REGEX: Lazy<Regex> = Lazy::new(|| Regex::new(r"\d+").unwrap());
    static COLON_REGEX: Lazy<Regex> = Lazy::new(|| Regex::new(r":").unwrap());
    static SEMICOLON_REGEX: Lazy<Regex> = Lazy::new(|| Regex::new(r";").unwrap());

    if INTEGER_REGEX.is_match(token) {
        Node::Number(token.parse::<i64>().unwrap())
    }
    else if COLON_REGEX.is_match(token) {
        Node::StartDefinition
    }
    else if SEMICOLON_REGEX.is_match(token) {
        Node::EndDefinition
    }
    else {
        Node::Word(token.to_string())
    }
}

fn parser(tokens: Vec<&str>) -> Vec<Node> {
    tokens.iter().map(|x| parse_token(*x)).collect()
}

fn eval(ctx: &mut Context, word: &Node) {
    match word {
        Node::Number(_) => {ctx.stack.push(word.clone());},
        Node::Word(word) => {
            if ctx.dict.contains_key(word) {
                let w = ctx.dict.get(word).unwrap().clone();
                eval(ctx, &w);
            }
            else {
                println!("{:?}", word);
                panic!();
            }
        },
        Node::Function(f) => {
            f(ctx);
        },
        Node::Proc(proc) => {
            eval_program(ctx, proc);
        },
        _ => panic!(),
    }

}

fn eval_program(ctx: &mut Context, program: &Vec<Node>) {
    let mut iter = program.iter();
    loop {
        let word = iter.next();
        if word.is_none() {
            break;
        }
        let word = word.unwrap();
        match word {
            Node::StartDefinition => {
                let key = iter.next().unwrap().get_word().unwrap();
                let mut proc: Vec<Node> = Vec::new();
                loop {
                    let n = iter.next().unwrap();
                    if n == &Node::EndDefinition {
                        break;
                    }
                    proc.push(n.clone());
                }
    
                ctx.dict.insert(key, Node::Proc(proc));
            }
            _ => {eval(ctx, &word)},
        };

    }
}


fn main() {
    let mut context = Context::new();
    context.register_function("+", |ctx: &mut Context|{
        let a = ctx.stack.pop().unwrap().get_number().unwrap();
        let b = ctx.stack.pop().unwrap().get_number().unwrap();
        ctx.stack.push(Node::Number(a+b));
    });
    context.register_function("*", |ctx: &mut Context|{
        let a = ctx.stack.pop().unwrap().get_number().unwrap();
        let b = ctx.stack.pop().unwrap().get_number().unwrap();
        ctx.stack.push(Node::Number(a*b));
    });

    context.register_function("dup", |ctx: &mut Context|{
        ctx.stack.push(ctx.stack.last().unwrap().clone());
    });

    context.register_function(".", |ctx: &mut Context|{
        println!("{:?}", ctx.stack.last().unwrap());
    });

    let code = ": square dup * ; 2 square .";
    //
    //let code = "fizz?  3 mod 0 = dup if .\" Fizz\" then ;";
    println!("running: {}", code);
    eval_program(&mut context, &parser(lexer(code)));
}
