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
      gst_all_1.gstreamer
      gst_all_1.gstreamer.dev
      gst_all_1.gst-plugins-base
      gst_all_1.gst-plugins-good
      gst_all_1.gst-libav
      gst_all_1.gst-vaapi
    ];
    LD_LIBRARY_PATH = lib.makeLibraryPath buildInputs;
  }
