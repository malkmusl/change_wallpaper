{ lib, buildInputs ? [], cargoBuildInputs ? [], stdenv, fetchFromGitHub }:


let
  pkgs = import <nixpkgs> {};
  rustPlatform = pkgs.rustPlatform;
in

stdenv.mkDerivation rec {
  pname = "change_wallpaper";
  version = "0.1.0";

  src = ./.;

  nativeBuildInputs = [ lib ]; # You can add additional native build inputs if needed

  buildInputs = buildInputs ++ [
    pkgs.rust
    pkgs.hyprland
  ];

  cargoBuildInputs = cargoBuildInputs ++ [
    # Add any additional cargo dependencies here if needed
  ];

  cargoBuildFlags = [
    "--release"
  ];

  meta = with lib; {
    description = "A simple Rust program to change wallpapers";
    license = licenses.mit;
  };

  installPhase = ''
    mkdir -p $out/bin
    cp target/release/change_wallpaper $out/bin/
  '';

  buildPhase = ''
    cargo build --release
  '';
}
