{
  description = "A basic flake with a shell";
  inputs.nixpkgs.url = "github:NixOS/nixpkgs/nixpkgs-unstable";
  inputs.flake-utils.url = "github:numtide/flake-utils";

  outputs = { self, nixpkgs, flake-utils }: 
    flake-utils.lib.eachDefaultSystem (system: let
      pkgs = nixpkgs.legacyPackages.${system};
    in {
      devShells.default = pkgs.mkShell {
          LIBCLANG_PATH = "${pkgs.llvmPackages.libclang}/lib/libclang.so";

          # necessary to override nix's defaults which cannot be overriden as others are
          shellHook = ''
            export CC="${pkgs.clang}/bin/clang"
            export CXX="${pkgs.clang}/bin/clang++"
            export LIBCLANG_PATH="${pkgs.libclang.lib}/lib"
            rustup override set stable
            '';
        nativeBuildInputs = [ pkgs.bashInteractive ];
        buildInputs = with pkgs; [
          openssl
          libclang
          clang
        ];
      };
    });
}
