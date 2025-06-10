# rust-tinynix

## Description

My random attempt at writing a toy tiny nixlang compiler in Rust, with the main
purpose of writing `nix!` macros.

This project is still WIP, so far the following syntax/semantics have been
implemented:

```rust
use rust_tinynix::nix;

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

            withStatement = with { a = 1; b = 2; c = 3; }; user;
            letInSimple = let x = 5; in x;
            letInBlock = let
              user1 = "alice";
              user2 = "bob";
            in {
              inherit user1 user2;
            };
            withLet = with let x = 5; in { a = x; b = x; }; a;
            letInWith = let a = 1; in with { a = 2; }; a;
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
```

## Questions

**Why**? Why not? life is short. Sing a song, dance in the rain, wear a skirt
or use nix in Rust. Who cares 🥺

**Should I use this in prod?** Yes please, and send me a message so we can
have a laugh together 🥺
