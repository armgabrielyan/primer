# Homebrew Formula for primer
#
# This formula is available via:
#   brew install armgabrielyan/tap/primer
#
# Or build from source:
#   brew install --HEAD armgabrielyan/tap/primer

class Primer < Formula
  desc "Primer CLI for AI-guided project recipes and milestone workflows"
  homepage "https://github.com/armgabrielyan/primer"
  license "MIT"
  version "0.2.0" # x-release-please-version

  on_macos do
    on_intel do
      url "https://github.com/armgabrielyan/primer/releases/download/v#{version}/primer-#{version}-x86_64-apple-darwin.tar.gz"
      sha256 "0e6b1b5500e3816256d991f4dc07040c73371578dc13d71b5ef38242eecfbcf0"
    end

    on_arm do
      url "https://github.com/armgabrielyan/primer/releases/download/v#{version}/primer-#{version}-aarch64-apple-darwin.tar.gz"
      sha256 "3f3c18908506cab829237970dd9fb1bcf58c651f698331fe7d9e368ef6ad760f"
    end
  end

  on_linux do
    on_intel do
      url "https://github.com/armgabrielyan/primer/releases/download/v#{version}/primer-#{version}-x86_64-unknown-linux-gnu.tar.gz"
      sha256 "9e8a3e0aae20b75525d80a4fbd7f2bc018a5220803f0ec09b555c834305f2631"
    end

    on_arm do
      url "https://github.com/armgabrielyan/primer/releases/download/v#{version}/primer-#{version}-aarch64-unknown-linux-gnu.tar.gz"
      sha256 "58886fbd719be53961da1f674371152ae4b1e3faab36038782dbd7e1b293ee5b"
    end
  end

  head do
    url "https://github.com/armgabrielyan/primer.git", branch: "main"
    depends_on "rust" => :build
  end

  def install
    if build.head?
      system "cargo", "install", *std_cargo_args
    else
      bin.install "primer"
    end
  end

  test do
    assert_match "primer", shell_output("#{bin}/primer --version")
  end
end
