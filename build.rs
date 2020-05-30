use lalrpop::Configuration;

fn main() {
    Configuration::new()
        .use_cargo_dir_conventions()
        .process_file("src/parser.lalrpop")
        .unwrap();
    println!("cargo:rerun-if-changed=src/parser.lalrpop");
}
