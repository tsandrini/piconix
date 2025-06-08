use piconix::nix;

// TODO:s
// - evaluation
// - testing
// - error handling
// - functions
// - builtins
// - ideally, we'd ditch pest and directly parse using syn
// - implement thunks
// - derivations
fn main() {
    let nix_expression = nix!({
        nums = {
            simpleInt = 5;
            negInt = -42;
            simpleFloat = 3.2121;
            negFloat = -2.7;
        };
        positive = true;
        negative = false;
        strings = {
            simple = "dlrow";
            interpolated = "hello ${world}!";
            escaped = "hello ''${world}! + escaped quotes \"\"\" hehe";
        };
        list = [ 1 2 3 "four" true false ];
        emtpyList = [];
        attrset = {
            deeplyNested = {
                a = 1;
                b = 2;
                c = 3;
            };
        };
        empty = {};
    });
    println!("{:#?}", nix_expression);
}
