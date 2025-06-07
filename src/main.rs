use rust_nix_macro::nix;

fn main() {
    let nix_expression = nix!({
        a = 1;
        world = "dlrow";
        b = "hello ${world}!";
        c = [1 2 3];
        escaped = "hello ''${world}! + escaped quotes \"\"\" hehe";
        d = {
            e = "nested";
            f = 42;
            deep = {
                g = "deeply nested";
                h = "another level";
            };
        };
        empty = {};
    });
    println!("{:#?}", nix_expression);
}
