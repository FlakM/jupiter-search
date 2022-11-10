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
            export LIBCLANG_PATH="${pkgs.libclang.lib}/lib"
            '';

        nativeBuildInputs = [ pkgs.bashInteractive ];
        buildInputs = with pkgs; [
          openssl # used for meilisearch
          llvmPackages.libclang
          llvmPackages.clang
          ffmpeg
        ] ++ lib.optionals stdenv.isDarwin (with darwin.apple_sdk.frameworks; [
            AppKit
            ApplicationServices
            CoreVideo
            fixDarwinDylibNames
            OpenGL
            Security
            Accelerate
          ]);
      };
    });
}
