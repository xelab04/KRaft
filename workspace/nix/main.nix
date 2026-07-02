{ pkgs ? import <nixpkgs> {} }:


let
  uid = "1000";
  gid = "1000";
  username = "kraft";

  # ttydFixed = pkgs.ttyd.override {
  #   libuv = pkgs.libuv;
  # };

  passwdFile = pkgs.writeTextDir "etc/passwd" ''
    root:x:0:0:root:/root:/bin/bash
    ${username}:x:${uid}:${gid}:Kraft Workspace User:/home/${username}:/bin/bash
  '';

  groupFile = pkgs.writeTextDir "etc/group" ''
    root:x:0:
    ${username}:x:${gid}:
  '';

  homeDirectory = pkgs.runCommand "home-dir" {} ''
    mkdir -p $out/home/${username}/.config
    mkdir -p $out/home/${username}/.config/fish/completions

    cat << 'EOF' > $out/home/${username}/.vimrc
    " Syntax highlighting and filetype detection
    syntax on
    filetype plugin indent on

    " Color scheme
    colorscheme desert

    " Indentation for YAML files
    autocmd FileType yaml setlocal ts=2 sts=2 sw=2 expandtab

    " UI Polish
    set number          " Show line numbers so you can find manifest errors easily
    set cursorline      " Highlight the current line you are editing
    set ruler            " Show the cursor position
    set showmatch       " Highlight matching brackets/braces
    EOF

    cat << 'EOF' > $out/home/${username}/.bashrc
    if [ -f /etc/bash_completion ]; then . /etc/bash_completion; fi

    source <(kubectl completion bash)
    alias kubectl=kubecolor
    complete -o default -F __start_kubectl kubecolor
    alias k=kubectl
    complete -o default -F __start_kubectl k
    if [ -f ~/.config/welcome ]; then ~/.config/welcome; fi
    EOF

    cat << 'EOF' > $out/home/${username}/.config/fish/config.fish
    set -g fish_greeting
    alias kubectl="kubecolor"
    alias k="kubecolor"
    #if test -f ~/.config/welcome
    #    ~/.config/welcome
    #end
    EOF

    cat << 'EOF' > $out/home/${username}/.config/welcome
    #!/bin/bash
    BLUE='\033[1;34m'
    GREEN='\033[1;32m'
    CYAN='\033[1;36m'
    YELLOW='\033[1;33m'
    RESET='\033[0m'
    BOLD='\033[1m'

    echo -e ""
    echo -e "  ''${BLUE}┌──────────────────────────────────────────────────────────┐''${RESET}"
    echo -e "  ''${BLUE}│''${RESET}                     ''${BOLD}KRaft Workspace''${RESET}                      ''${BLUE}│''${RESET}"
    echo -e "  ''${BLUE}└──────────────────────────────────────────────────────────┘''${RESET}"
    echo -e ""
    echo -e "  ''${BOLD}INSTALLED TOOLS''${RESET}"
    echo -e "  ────────────────────────────────────────────────────────────"
    echo -e "  ''${GREEN}  Kubernetes: ''${RESET}   kubectl  |  kubecolor  |  k9s"
    echo -e "  ''${CYAN}  Utilities: ''${RESET}    git  |  fish  | wget  |  vim  |  iputils"
    echo -e "  ''${BLUE}  Nice To Have: ''${RESET} k (alias)  |  kubectl autocompletion"
    echo -e "  ────────────────────────────────────────────────────────────"
    echo -e "    Type ''${CYAN}fish''${RESET} to switch shells, or get started with ''${GREEN}k get nodes''${RESET}"
    echo -e ""
    EOF

    chmod +x $out/home/${username}/.config/welcome

    ${pkgs.kubectl}/bin/kubectl completion fish > $out/home/${username}/.config/fish/completions/kubectl.fish
        ln -s kubectl.fish $out/home/${username}/.config/fish/completions/k.fish
  '';

  # welcomeScript = pkgs.writeScriptDir "home/${username}/.config/welcome" ''

  #   '';

in
pkgs.dockerTools.buildLayeredImage {
  name = "kraft-workspace";
  tag = "latest";
  maxLayers = 100;

  # copyToRoot
  contents = with pkgs; [
    coreutils
    bash
    bashInteractive
    # bash-completion
    git
    wget
    vim
    iputils
    fish
    kubectl
    kubecolor
    k9s
    # neovim
    ttyd
    libuv
    libwebsockets
    # ttydFixed

    passwdFile
    groupFile
    homeDirectory
  ];

  fakeRootCommands = ''
    mkdir -p home/${username}/.config/fish
    mkdir -p home/${username}/.local/share/fish
    mkdir -p home/${username}/.local/state

    chown -R ${uid}:${gid} home/${username}
    chmod -R 755 home/${username}
  '';

  config = {
    User = "${uid}:${gid}";
    WorkingDir = "/home/${username}";

    ExposedPorts = {
      "7681/tcp" = {};
    };

    Env = [
      "PATH=/bin"
      "LD_LIBRARY_PATH=/lib:/usr/lib"
      "TERM=xterm-256color"
      "HOME=/home/${username}"
      "KUBE_EDITOR=vim"
    ];

    Entrypoint = ["${pkgs.ttyd}/bin/ttyd" "--writable" "bash"];
  };
}
