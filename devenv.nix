{
  pkgs,
  lib,
  config,
  inputs,
  ...
}: {
  # https://devenv.sh/basics/
  env = rec {
    LD_LIBRARY_PATH = lib.makeLibraryPath [
      pkgs.stdenv.cc.cc
      pkgs.libxml2
    ];
    NIX_LD_LIBRARY_PATH = LD_LIBRARY_PATH;
    NIX_LD = pkgs.runCommand "ld.so" {} ''
      ln -s "$(cat '${pkgs.stdenv.cc}/nix-support/dynamic-linker')" $out
    '';
  };

  # https://devenv.sh/packages/
  packages = with pkgs; [
    cargo-generate # generate rust projects from github templates
    cargo-udeps # find unused dependencies in Cargo.toml
    ldproxy

    # required for esp development
    espup # tool for installing esp-rs toolchain
    rustup # rust installer, required by espup
    espflash # flash binary to esp
    python3
  ];

  # https://devenv.sh/languages/
  # languages.rust.enable = true;

  # https://devenv.sh/processes/
  # processes.cargo-watch.exec = "cargo-watch";

  # https://devenv.sh/services/
  # services.postgres.enable = true;

  # https://devenv.sh/scripts/
  # scripts.hello.exec = ''
  #   echo hello from $GREET
  # '';

  enterShell = ''
    echo -e "\e[1mInstalling toolchains for esp"
    echo -e "-----------------------------\e[0m"
    espup install --export-file $DEVENV_ROOT/esp-export.sh
    source $DEVENV_ROOT/esp-export.sh
  '';

  # https://devenv.sh/tasks/
  # tasks = {
  #   "myproj:setup".exec = "mytool build";
  #   "devenv:enterShell".after = [ "myproj:setup" ];
  # };

  # https://devenv.sh/tests/
  # enterTest = ''
  #   echo "Running tests"
  #   git --version | grep --color=auto "${pkgs.git.version}"
  # '';

  # https://devenv.sh/git-hooks/
  # git-hooks.hooks.shellcheck.enable = true;

  # See full reference at https://devenv.sh/reference/options/
}
