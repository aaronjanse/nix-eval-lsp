(import "${fetchGit {
    url = "https://github.com/edolstra/flake-compat";
    rev = "99f1c2157fba4bfe6211a321fd0ee43199025dbf";
}}" { src = ./.; }).defaultNix.default