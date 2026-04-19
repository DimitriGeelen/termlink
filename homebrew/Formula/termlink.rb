# Homebrew formula for TermLink
# Cross-terminal session communication tool
#
# To use this formula from a tap:
#   brew tap DimitriGeelen/termlink https://github.com/DimitriGeelen/homebrew-termlink
#   brew install termlink

class Termlink < Formula
  desc "Cross-terminal session communication — message bus with terminal endpoints"
  homepage "https://github.com/DimitriGeelen/termlink"
  version "0.9.1"
  license "MIT"

  on_macos do
    if Hardware::CPU.arm?
      url "https://github.com/DimitriGeelen/termlink/releases/download/v#{version}/termlink-darwin-aarch64"
      sha256 "a78c227460ff976f02fcf19dcba92d11f2d190d2764d60121a40e06a2c13c301"  # Update after first release
    else
      url "https://github.com/DimitriGeelen/termlink/releases/download/v#{version}/termlink-darwin-x86_64"
      sha256 "7959755a80176fc990f930564061ca8b603cd65015ce97b208ff7215c4c8fac9"  # Update after first release
    end
  end

  on_linux do
    if Hardware::CPU.arm?
      url "https://github.com/DimitriGeelen/termlink/releases/download/v#{version}/termlink-linux-aarch64"
      sha256 "9f24ccde42dfd192927074fd791d5fc5f053ac7d8f5b07493b90c0f7c19b19c0"  # Update after first release
    else
      # Static musl build — works on both glibc and musl hosts, including
      # minimal LXC images where the gnu binary silently fails to load.
      # T-1135 (from T-1070 GO).
      url "https://github.com/DimitriGeelen/termlink/releases/download/v#{version}/termlink-linux-x86_64-static"
      sha256 "e5e0ded04d6e0c5d2257e844416ca7b296135fcad19c0309760abe41a7f2e288"  # Update after first release
    end
  end

  def install
    binary = Dir["termlink-*"].first || "termlink"
    bin.install binary => "termlink"
  end

  test do
    assert_match "termlink #{version}", shell_output("#{bin}/termlink --version")
  end
end
