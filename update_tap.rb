def main
  output_file = ARGV[0]
  version = ARGV[1]
  sha = ARGV[2]
  url = "https://github.com/Reeywhaar/tmbliss/releases/download/v#{version}/homebrew.zip"

  template_path = File.join(File.dirname(__FILE__), "tmbliss.rb.template")
  content = File.read(template_path)
  content = content.gsub("{{URL}}", url)
  content = content.gsub("{{VERSION}}", version)
  content = content.gsub("{{SHA256}}", sha)

  File.write(output_file, content)
end

main
