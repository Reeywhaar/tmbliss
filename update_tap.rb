def main
  file = ARGV[0]
  version = ARGV[1]
  sha = ARGV[2]
  url = "https://github.com/Reeywhaar/tmbliss/releases/download/v#{version}/homebrew.zip"

  content = File.read(file)
  content = content.gsub(/# url-placeholder\n.*$/, "# url-placeholder\n  url \"#{url}\"")
  content = content.gsub(/# version-placeholder\n.*$/, "# version-placeholder\n  version \"#{version}\"")
  content = content.gsub(/# sha256-placeholder\n.*$/, "# sha256-placeholder\n  sha256 \"#{sha}\"")
  content = content.gsub(/^\s+revision.*$/, "")

  File.open(file, "w") do |f|
    f.write(content)
  end
end

main