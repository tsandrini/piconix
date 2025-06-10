use rust_tinynix::nix;

fn main() {
    let nix_expression = nix!({
        eh = "henloo" + " worldl" + " :3";
        nums = 10 - 2 + 3;
    });
    println!("{:#?}", nix_expression);
}
