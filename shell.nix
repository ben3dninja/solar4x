{pkgs ? import <nixpkgs> {}}: let
  rust_overlay = import (builtins.fetchTarball "https://github.com/oxalica/rust-overlay/archive/master.tar.gz");
  pkgs = import <nixpkgs> {overlays = [rust_overlay];};
  rustVersion = "latest";
  #rustVersion = "1.62.0";
  rust = pkgs.rust-bin.stable.${rustVersion}.default.override {
    extensions = [
      "rust-src" # for rust-analyzer
      "rust-analyzer"
    ];
  };
in
  pkgs.mkShell rec {
    buildInputs =
      [
        rust
      ]
      ++ (
        with pkgs; [
          udev
          alsa-lib
          vulkan-loader
          xorg.libX11
          xorg.libXcursor
          xorg.libXi
          xorg.libXrandr # To use the x11 feature
          libxkbcommon
          wayland # To use the wayland feature

          taplo # toml language server
        ]
      );

    nativeBuildInputs = [
      pkgs.pkg-config
    ];

    LD_LIBRARY_PATH = pkgs.lib.makeLibraryPath buildInputs;
  }
