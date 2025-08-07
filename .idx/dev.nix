{pkgs}: {
  channel = "stable-24.05";
  packages = [
    pkgs.jdk19
    pkgs.unzip
    pkgs.rustup
    pkgs.cargo-expand
    pkgs.pkg-config
    pkgs.clang
    pkgs.libclang
    pkgs.cmake
    pkgs.protobuf
    pkgs.unzip
    pkgs.zip
  ];
  idx.extensions = [
    pkgs.gh
    "Codium.codium"
    "Dart-Code.dart-code"
    "Dart-Code.flutter"
    "denoland.vscode-deno"
    "golang.go"
    "Nash.awesome-flutter-snippets"
    "RodrigoRahman.flutter-dart-utils"
  ];
}