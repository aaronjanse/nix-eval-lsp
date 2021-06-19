**Use [nix-community/rnix-lsp](https://github.com/nix-community/rnix-lsp). The `nix-eval-lsp` repo is merely a proof-of-concept.**

This README has been shortened to encourage readers to use [nix-lsp](https://github.com/nix-community/rnix-lsp).  
Its previous contents, including a gif and usage instructions, are [available in git history](https://github.com/aaronjanse/nix-eval-lsp/tree/c48f47f705b361ea1d3c1f3e8c2bc3b4894a1793).

---

`nix-eval-lsp` is an proof-of-concept [language server](https://langserver.org/) for [Nix](https://nixos.org) that provides completions and tooltips by evaluating Nix expressions as they are typed. This allows for several features:

- hover over an expression to see its value
- auto-complete inside expressions such as `with pkgs; [ <typing here> ]`
- goto definitions across files

The evaluator is developed for the purpose of debugging only. It does not aim to correctly implement Nix in its entirety.

Huge thank you to [@jd91mZM2](https://github.com/jd91mZM2) for [nix-community/rnix-parser](https://github.com/nix-community/rnix-parser) and [nix-community/rnix-lsp](https://github.com/nix-community/rnix-lsp).
