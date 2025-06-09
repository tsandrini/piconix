use rust_tinynix::nix;

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
    let nix_expression = nix!(rec {
        basicExample = {
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
                interpolated = "hello ${simple}!";
                interpolatedDotted = "hello.${basicExample.strings.simple}!";
                escaped = "hello ''${world}! + escaped quotes \"\"\" hehe";
            };
            list = [ 1 2 3 "four" true false null ];
            attrList = [ { a = 1; b = 2; c = 3; } { a = 4; b = 5; c = 6; } ];
            emtpyList = [];
            emptyAttrSet = {};
        };
        paths = {
            nixpkgs = <nixpkgs>;
            bin = /bin;
            home = ~/.;
            local = ./src/main.rs;
            localPrev = ../piconix;
        };
        keywords = rec {
            inherit user;
            inherit (config.services.myService) enable configFile;

            letInSimple = let x = 5; in x;
            letInBlock = let
              user1 = "alice";
              user2 = "bob";
            in {
              inherit user1 user2;
            };
        };
        config = {
            services.myService.enable = true;
            services.myService.configFile = null;

            services.myService.config = {
                enableSomething = true;
                configOption = "config.txt";
                package = inputs.flake.packages.myPackage;
            };

            services."name with weird symbols !@#$%^&*() and spaces".enable = true;
            services."${basicExample}".enable = true; # TODO fix this
            services.myOtherService.enable = config.services.myService.enable;
        };
    });
    println!("{:#?}", nix_expression);
}
