use rust_nix_macro::nix;

fn main() {
    let nix_expression = nix!({
        str1 = "doublequoted escape \" should work";
        str2 = "and also interpolation escaping ''${blah}";
    });

    println!("{:#?}", nix_expression);
}
