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
  version "0.1.0" # x-release-please-version

  on_macos do
    on_intel do
      url "https://github.com/armgabrielyan/primer/releases/download/v#{version}/primer-#{version}-x86_64-apple-darwin.tar.gz"
      sha256 "REPLACE_X86_64_DARWIN_SHA256"
    end

    on_arm do
      url "https://github.com/armgabrielyan/primer/releases/download/v#{version}/primer-#{version}-aarch64-apple-darwin.tar.gz"
      sha256 "REPLACE_AARCH64_DARWIN_SHA256"
    end
  end

  on_linux do
    on_intel do
      url "https://github.com/armgabrielyan/primer/releases/download/v#{version}/primer-#{version}-x86_64-unknown-linux-gnu.tar.gz"
      sha256 "REPLACE_X86_64_LINUX_SHA256"
    end

    on_arm do
      url "https://github.com/armgabrielyan/primer/releases/download/v#{version}/primer-#{version}-aarch64-unknown-linux-gnu.tar.gz"
      sha256 "REPLACE_AARCH64_LINUX_SHA256"
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
