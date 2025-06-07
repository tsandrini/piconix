use rust_nix_macro::nix;

fn main() {
    let nix_expression = nix!({
        x = 10;
        eh = x;
        uh = [ 3 4 6 ];
        m = {
            l = 10;
        };
    });

    println!("{:#?}", nix_expression);
}
