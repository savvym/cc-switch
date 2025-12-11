class CcSwitch < Formula
  desc "CLI tool for managing AI provider configurations"
  homepage "https://github.com/farion1231/cc-switch"
  version "3.8.2"

  if OS.mac? && Hardware::CPU.intel?
    url "https://github.com/farion1231/cc-switch/releases/download/cli-v3.8.2/cc-switch-macos-x86_64.tar.gz"
    sha256 "TO_BE_FILLED_AFTER_RELEASE"
  elsif OS.mac? && Hardware::CPU.arm?
    url "https://github.com/farion1231/cc-switch/releases/download/cli-v3.8.2/cc-switch-macos-aarch64.tar.gz"
    sha256 "TO_BE_FILLED_AFTER_RELEASE"
  elsif OS.linux?
    url "https://github.com/farion1231/cc-switch/releases/download/cli-v3.8.2/cc-switch-linux-x86_64.tar.gz"
    sha256 "TO_BE_FILLED_AFTER_RELEASE"
  end

  def install
    bin.install "cc-switch"
  end

  test do
    system "#{bin}/cc-switch", "--version"
  end
end
