{ pkgs ? (import <nixpkgs> { 
    config.allowUnfree = true;
}), ... }:

pkgs.mkShell {
  nativeBuildInputs = with pkgs; [
    rustup 
    rust-analyzer
    pkg-config
    openssl
    alsaLib
    udev
  ];

  shellHook = ''
    export LD_LIBRARY_PATH=${pkgs.udev}/lib:$LD_LIBRARY_PATH
    export LD_LIBRARY_PATH=${pkgs.alsaLib}/lib:$LD_LIBRARY_PATH
    export LD_LIBRARY_PATH=${pkgs.xorg.libX11}/lib:$LD_LIBRARY_PATH
    export LD_LIBRARY_PATH=${pkgs.xorg.libXcursor}/lib:$LD_LIBRARY_PATH
    export LD_LIBRARY_PATH=${pkgs.xorg.libXrandr}/lib:$LD_LIBRARY_PATH
    export LD_LIBRARY_PATH=${pkgs.xorg.libXi}/lib:$LD_LIBRARY_PATH
    export LD_LIBRARY_PATH=${pkgs.vulkan-loader}/lib:$LD_LIBRARY_PATH
  '';
}
