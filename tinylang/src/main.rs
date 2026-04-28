pub mod ast;
pub mod interpreter;

use lalrpop_util::lalrpop_mod;
lalrpop_mod!(
    #[allow(clippy::ptr_arg)]
    #[rustfmt::skip]
    tinylang
);

use interpreter::Interpreter;

fn main() {
    let source: &str = r#"
def greet(name)
    puts "Hello, " + name + "!";
end;

greet("world");

x = 10;
y = 3;
puts x + y;

def factorial(n)
    if n < 2 then
        return 1;
    end;
    return n * factorial(n - 1);
end;

result = factorial(5);
puts result;

i = 1;
while i < 6 do
    puts i;
    i = i + 1;
end;

fetch("google.com");

if 10 > 5 then
    puts "ten is greater";
else
    puts "this won't print";
end;
"#;

    let parser = tinylang::ProgramParser::new();
    let program = parser.parse(source).unwrap_or_else(|e| {
        eprintln!("Parse error: {e}");
        std::process::exit(1);
    });

    let mut interp = Interpreter::new();
    if let Err(e) = interp.run(program) {
        eprintln!("Runtime error: {e}");
        std::process::exit(1);
    }
}