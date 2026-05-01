cask "llm-proxy-manager" do
  version "4.1.32"
  sha256 :no_check

  name "llm-proxy-Manager"
  desc "Professional Account Management for AI Services"
  homepage "https://github.com/liumenglife/llm-proxy-manager"

  on_macos do
    url "https://github.com/liumenglife/llm-proxy-manager/releases/download/v#{version}/llm-proxy-manager_#{version}_universal.dmg"

    app "llm-proxy-Manager.app"

    zap trash: [
      "~/Library/Application Support/com.llmproxy.llm-proxy-manager",
      "~/Library/Caches/com.llmproxy.llm-proxy-manager",
      "~/Library/Preferences/com.llmproxy.llm-proxy-manager.plist",
      "~/Library/Saved Application State/com.llmproxy.llm-proxy-manager.savedState",
    ]

    caveats <<~EOS
      If you encounter the "App is damaged" error, please run the following command:
        sudo xattr -rd com.apple.quarantine "/Applications/llm-proxy-Manager.app"

      Or install with the --no-quarantine flag:
        brew install --cask --no-quarantine llm-proxy-manager
    EOS
  end

  on_linux do
    arch arm: "aarch64", intel: "amd64"

    url "https://github.com/liumenglife/llm-proxy-manager/releases/download/v#{version}/llm-proxy-manager_#{version}_#{arch}.AppImage"
    binary "llm-proxy-manager_#{version}_#{arch}.AppImage", target: "llm-proxy-manager"

    preflight do
      system_command "/bin/chmod", args: ["+x", "#{staged_path}/llm-proxy-manager_#{version}_#{arch}.AppImage"]
    end
  end
end
