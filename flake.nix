{
	description = "Home automation system, services and ui/interfaces";

  inputs = {
    nixpkgs.url = "github:NixOS/nixpkgs/nixos-unstable";
    flake-utils.url = "github:numtide/flake-utils";
    rust-overlay = {
      url = "github:oxalica/rust-overlay";
      inputs = {
        nixpkgs.follows = "nixpkgs";
      };
    };
  };

  outputs = { self, nixpkgs, flake-utils, rust-overlay }:
    flake-utils.lib.eachDefaultSystem
      (system:
        let
          overlays = [ (import rust-overlay) ];
          pkgs = import nixpkgs { inherit system overlays; };
          rustToolchain = pkgs.rust-bin.fromRustupToolchainFile ./rust-toolchain.toml;
          rust = pkgs.makeRustPlatform {
            cargo = rustToolchain;
            rustc = rustToolchain;
          };

		  src = ./.;
		  cargoLock = {
			  lockFile = "${src}/Cargo.lock";
			  outputHashes = {
				  "bme280-0.5.1" = "sha256-EV1V6XeYxjoIwI/mzTJp1CZriYKucCr14SkRWC8DaLE=";
				  "byteseries-0.7.1" = "sha256-Gz+uj56L5iCL6gHSEq9uFYLKYUaMXKkhWx8RhNKH77c=";
				  "colorimetry-0.0.2" = "sha256-Olxp0GMmUDDfeUiTusScwAidY5POOoe45K8ezAojN4Q=";
				  "dbstruct-0.6.0" = "sha256-3ACZN0+MJe3r9l/0/Lif3004D8NlBYs56gH3UQWqotY=";
				  "tui-tree-widget-0.23.0" = "sha256-rVHmuBg6x592pNBTzjVbzQ87J8HlbTAY9vz8ZPJwmYc=";
			  };
		  };

          ####################################################################
          #### text widget package                                        ####
          ####################################################################
          text-widget = with pkgs; let
            cargoTOML = lib.importTOML "${src}/crates/text-widget/Cargo.toml";
			inherit rustToolchain rust;
          in 
			  rust.buildRustPackage
				{
				  pname = cargoTOML.package.name;
				  version = cargoTOML.package.version;

				  inherit src cargoLock;

				  meta = {
					inherit (cargoTOML.package) description homepage;
					maintainers = cargoTOML.package.authors;
				  };
				};


          ####################################################################
          #### text widget package                                        ####
          ####################################################################
          ui = with pkgs; let
            cargoTOML = lib.importTOML "${src}/crates/ui/Cargo.toml";
			inherit rustToolchain rust;
          in 
			  rust.buildRustPackage
				{
				  pname = cargoTOML.package.name;
				  version = cargoTOML.package.version;

				  inherit src cargoLock;

				  meta = {
					inherit (cargoTOML.package) description homepage;
					maintainers = cargoTOML.package.authors;
				  };
				};

          ####################################################################
          #### dev shell                                                  ####
          ####################################################################
          devShell = with pkgs;
            mkShell {
              name = "ha";
              inputsFrom = [ text-widget ];
              RUST_SRC_PATH = "${rustPlatform.rustLibSrc}";
              CARGO_TERM_COLOR = "always";
            };
        in
        {
		  ui = ui;
          devShells.default = devShell;
		  defaultPackage = text-widget;
        }) // {
			overlays.default = _: prev: {
				text-widget = self.defaultPackage.${prev.system};
				ui = self.${prev.system};
			};
		};
}
