class Textstep < Formula
  desc "TUI step sequencer, drum machine, and synthesizer"
  homepage "https://github.com/illobo/textStep"
  version "1.2.5"
  license "MIT"

  on_macos do
    url "https://github.com/illobo/textStep/releases/download/v#{version}/textstep-universal-apple-darwin.tar.gz"
    sha256 "bc937273465ad760699bbaacac2071f93bbb84317fe3cf7ec82a1934e7ad6ea3"
  end

  on_linux do
    url "https://github.com/illobo/textStep/releases/download/v#{version}/textstep-x86_64-unknown-linux-gnu.tar.gz"
    sha256 "64d0eb9aa1a79c3b419729c0de21b3182e65bc2791b4c1d96a11e26e35d75535"
  end

  def install
    bin.install "textstep"
  end

  test do
    assert_match "textstep #{version}", shell_output("#{bin}/textstep --version")
  end
end
