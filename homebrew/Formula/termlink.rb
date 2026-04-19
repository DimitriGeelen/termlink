# Homebrew formula for TermLink
# Cross-terminal session communication tool
#
# To use this formula from a tap:
#   brew tap DimitriGeelen/termlink https://github.com/DimitriGeelen/homebrew-termlink
#   brew install termlink

class Termlink < Formula
  desc "Cross-terminal session communication — message bus with terminal endpoints"
  homepage "https://github.com/DimitriGeelen/termlink"
  version "0.9.0"
  license "MIT"

  on_macos do
    if Hardware::CPU.arm?
      url "https://github.com/DimitriGeelen/termlink/releases/download/v#{version}/termlink-darwin-aarch64"
      sha256 "PLACEHOLDER_SHA256_AARCH64"  # Update after first release
    else
      url "https://github.com/DimitriGeelen/termlink/releases/download/v#{version}/termlink-darwin-x86_64"
      sha256 "PLACEHOLDER_SHA256_X86_64"  # Update after first release
    end
  end

  on_linux do
    if Hardware::CPU.arm?
      url "https://github.com/DimitriGeelen/termlink/releases/download/v#{version}/termlink-linux-aarch64"
      sha256 "PLACEHOLDER_SHA256_LINUX_AARCH64"  # Update after first release
    else
      # Static musl build — works on both glibc and musl hosts, including
      # minimal LXC images where the gnu binary silently fails to load.
      # T-1135 (from T-1070 GO).
      url "https://github.com/DimitriGeelen/termlink/releases/download/v#{version}/termlink-linux-x86_64-static"
      sha256 "PLACEHOLDER_SHA256_LINUX_X86_64_STATIC"  # Update after first release
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
