**Use [nix-community/rnix-lsp](https://github.com/nix-community/rnix-lsp). The `nix-eval-lsp` repo is merely a proof-of-concept.**

`nix-eval-lsp` is an proof-of-concept [language server](https://langserver.org/) for [Nix](https://nixos.org) that provides completions and tooltips by evaluating Nix expressions as they are typed. This allows for several features:

- hover over an expression to see its value
- auto-complete inside expressions such as `with pkgs; [ <typing here> ]`
- goto definitions across files

The evaluator is developed for the purpose of debugging tools only. It does not aim to correctly implement Nix in its entirety.

Huge thank you to [@jd91mZM2](https://github.com/jd91mZM2) for [nix-community/rnix-parser](https://github.com/nix-community/rnix-parser) and [nix-community/rnix-lsp](https://github.com/nix-community/rnix-lsp).
