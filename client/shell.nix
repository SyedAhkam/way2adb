with import <nixpkgs> {
  config = {
    android_sdk.accept_license = true;
    allowUnfree = true;
  };
};

let
  buildToolsVersion = "33.0.3";
  androidComposition = androidenv.composeAndroidPackages {
    buildToolsVersions = [
      buildToolsVersion
    ];
    platformVersions = [ "33" ];
    abiVersions = [
      "x86_64"
    ];
  };
in
mkShell rec {
  ANDROID_SDK_ROOT = "${androidComposition.androidsdk}/libexec/android-sdk";
  GRADLE_OPTS = "-Dorg.gradle.project.android.aapt2FromMavenOverride=${ANDROID_SDK_ROOT}/build-tools/${buildToolsVersion}/aapt2";

  buildInputs = [
    androidComposition.androidsdk
    jdk17
    kotlin-language-server
  ];
}
