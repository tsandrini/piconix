use piconix::nix;

fn main() {
    let nix_expression = nix!({
        myvalue = config.services.myservices.myvalue;
        lol = null;
    });
    println!("{:#?}", nix_expression);
}
