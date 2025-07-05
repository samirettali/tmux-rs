{pkgs ? import <nixpkgs> {}}: let
  # Import fenix for newer rust versions
  fenix = import (fetchTarball "https://github.com/nix-community/fenix/archive/main.tar.gz") {};
in
  pkgs.mkShell {
    buildInputs = with pkgs; [
      # Build tools - use fenix for newer rust
      fenix.complete.toolchain
      pkg-config

      # Required libraries for tmux
      libevent
      ncurses

      # Development tools
      gdb
      lldb
    ];

    # Environment variables for building
    shellHook = ''
      export PKG_CONFIG_PATH="${pkgs.libevent}/lib/pkgconfig:${pkgs.ncurses}/lib/pkgconfig:$PKG_CONFIG_PATH"
      export LIBRARY_PATH="${pkgs.libevent}/lib:${pkgs.ncurses}/lib:$LIBRARY_PATH"
      export C_INCLUDE_PATH="${pkgs.libevent}/include:${pkgs.ncurses}/include:$C_INCLUDE_PATH"

      # Set the correct Rust target for macOS ARM
      export CARGO_BUILD_TARGET="aarch64-apple-darwin"

      echo "Development environment ready!"
      echo "Libraries available:"
      echo "- libevent: ${pkgs.libevent}"
      echo "- ncurses: ${pkgs.ncurses}"
      echo "- rust version: $(rustc --version)"
    '';
  }
