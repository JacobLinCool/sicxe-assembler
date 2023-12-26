use sicxe::assembler::assemble;

fn main() {
    let args: Vec<String> = std::env::args().collect();
    let filename = &args.get(1).expect("No filename given");

    let source = std::fs::read_to_string(filename).expect("Failed to read file");
    let obj = assemble(&source);
    if let Err(e) = obj {
        println!("{}", e);
        return;
    }

    println!("{}", obj.unwrap());
}
