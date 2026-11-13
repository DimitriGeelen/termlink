# Homebrew formula for TermLink
# Cross-terminal session communication tool
#
# To use this formula from a tap:
#   brew tap DimitriGeelen/termlink https://github.com/DimitriGeelen/homebrew-termlink
#   brew install termlink
#
# This file is the TEMPLATE. Do not edit version or sha256 by hand (T-3754): on every
# vX.Y.Z tag, the release workflow's homebrew-tap job runs homebrew/render-formula.sh
# with the release's checksums.txt and pushes the result to the tap repo. The values
# below are the last ones rendered into this repo (v0.12.11) and may lag the tap.

class Termlink < Formula
  desc "Cross-terminal session communication — message bus with terminal endpoints"
  homepage "https://github.com/DimitriGeelen/termlink"
  version "0.12.11"
  license "Apache-2.0"

  on_macos do
    if Hardware::CPU.arm?
      url "https://github.com/DimitriGeelen/termlink/releases/download/v#{version}/termlink-darwin-aarch64"
      sha256 "e85be6ce15f3cc13e44adcdebdf4cd583988529e6e6595ae8243fe15ba14a60e"
    else
      url "https://github.com/DimitriGeelen/termlink/releases/download/v#{version}/termlink-darwin-x86_64"
      sha256 "e3f90cd73f671877881c2af07551eabcf3d52899048d970fa354c7064c83bf13"
    end
  end

  on_linux do
    if Hardware::CPU.arm?
      url "https://github.com/DimitriGeelen/termlink/releases/download/v#{version}/termlink-linux-aarch64"
      sha256 "19bf022bfaffbdbccd0d74cbf26ed10096a7692a129d87cb3fb330ce593adf74"
    else
      # Static musl build — works on both glibc and musl hosts, including
      # minimal LXC images where the gnu binary silently fails to load.
      # T-1135 (from T-1070 GO).
      url "https://github.com/DimitriGeelen/termlink/releases/download/v#{version}/termlink-linux-x86_64-static"
      sha256 "a30545a243d2a67463be39e03b17a1b410ad005709f3f0af052b0b8818c963d5"
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
