{
  pkgs ? import <nixpkgs> { },
}:

with pkgs;

mkShell.override
  {
    stdenv = clangStdenv;
  }
  rec {
    nativeBuildInputs = [
      pkg-config
    ];
    buildInputs = [
      stdenv.cc.cc
      llvmPackages.libclang
      pipewire
      clang-tools
      x264
    ];
    LD_LIBRARY_PATH = lib.makeLibraryPath buildInputs;
  }
