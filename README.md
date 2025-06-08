# rust-tinynix

## Description

My random attempt at writing a toy tiny nixlang compiler in Rust, with the main
purpose of writing `nix!` macros such as

TODO: A random example showcasing what kind of syntax and flow this project
is trying to achieve, some of it is already supported and some of it isn't.

```rust
use rust_tinynix::{nix, NixExpr, NixValue, NixStringPart, Scope, EvaluationError};
use indexmap::{indexmap, IndexMap};

use rust_tinynix::{nix, NixExpr, NixValue, Scope};

fn main() {
    // 🍊 Define native Rust variables.
    let domain_name = "my-cool-project.dev";
    let admin_email = "admin@my-cool-project.dev";

    // 🚀 Use the `nix!` macro to write a Nix expression as a function.
    // This showcases how configuration can be parameterized.
    let nix_config_function = nix!(
        # This is a native comment in nix!
        # A Nix function accepting `pkgs` and `lib` with default values.
        { pkgs ? <nixpkgs>, lib ? pkgs.lib }:

        rec { # ✨ A recursive set makes self-reference clean and easy.
            networking = {
                hostName = "server1";
                domain = domain_name_from_rust; # <-- Use Rust variables.
            };

            services.nginx = {
                enable = true;
                package = pkgs.nginx; # <-- Use arguments like `pkgs`.
                httpPort = 80;
                virtualHosts.primary = {
                    serverName = networking.domain; # <-- `rec` makes this possible.
                    adminEmail = admin_email_from_rust;
                };
            };

            users.list = [ "alice" "bob" ];

            # Call a function from a Nix library!
            motd = lib.strings.toUpper "welcome to ${networking.hostName}";
        }
    );

    // 📞 "Call" the Nix function from Rust and evaluate it.
    let scope: Scope = indexmap! {
        "domain_name_from_rust" => NixExpr::Value(NixValue::String(domain_name.to_string())),
        "admin_email_from_rust" => NixExpr::Value(NixValue::String(admin_email.to_string())),
    };
    println!("--- Evaluating Nix expression ---");
    let nix_evaluated = rust_tinynix::nix_eval(&nix_config_function, &scope)
        .expect("Evaluation failed");
    println!("✅ Evaluation complete!");

    //  manipulating the result in Rust
    println!("\n--- Manipulating Nix data in Rust ---");
    let mut users: Vec<String> = extract_users(&nix_evaluated);

    println!("Users from Nix: {:?}", users);
    users.push("charlie"); // Add a new user!
    println!("Updated users in Rust: {:?}", users);

    // 🤯 Generate a NEW Nix expression from our Rust data.
    // This shows the two-way data flow between Rust and Nix.
    println!("\n--- Generating a new Nix expression ---");
    let deployment_script = nix!({
        description = "Deployment script for ${domain_name}";
        tasks = [
            "1. Pull latest container from registry"
            "2. Stop old container"
            "3. Start new container"
            "4. Notify users that deployment is complete"
        ];
        usersToNotify = users; # <-- Pass the updated Rust vector back to Nix!
        domain_name = domain_name;
    });

    println!("Generated deployment script AST:\n{:#?}", deployment_script);
}
```

## Questions

**Why**? Why not? life is short. Sing a song, dance in the rain, wear a skirt
or use nix in Rust. Who cares 🥺

**Should I use this in prod?** Yes please, and send me a message so we can
have a laugh together 🥺
