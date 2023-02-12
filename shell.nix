# SPDX-FileCopyrightText: 2020-2021 Katharina Fey <kookie@spacekookie.de>
#
# SPDX-License-Identifier: GPL-3.0-or-later

with import <nixpkgs> {};
stdenv.mkDerivation {
  name = "rust-dev";
  buildInputs = [ 
    # core build tools
    rustc cargo rustfmt rust-analyzer clangStdenv
  ];
}
