# Read each year's fortunes into `site.data.fortunes`, newest year first, for
# fortunes.md to render. Entries are separated by a line holding only `%`, and
# an empty entry fails the build. `dat` is nil until `make fortunes` has run.

module Jekyll
  class FortunesGenerator < Generator
    def generate(site)
      dir = File.join(site.source, 'fortunes')
      files = Dir.glob(File.join(dir, '[0-9][0-9][0-9][0-9].txt')).sort.reverse

      site.data['fortunes'] = files.map do |file|
        year = File.basename(file, '.txt')
        {
          'year' => year,
          'path' => "/fortunes/#{year}.txt",
          'dat' => (File.exist?("#{file}.dat") ? "/fortunes/#{year}.txt.dat" : nil),
          'entries' => parse(file, "fortunes/#{year}.txt"),
        }
      end
    end

    private

    def parse(file, name)
      # strfile only splits on a bare `%`, so one with trailing whitespace or
      # a CRLF ending would merge two fortunes there. Fail instead of quietly
      # showing them as one.
      File.read(file).split("\n").each_with_index do |line, n|
        if line.match?(/\A%\s+\z/)
          raise Errors::FatalException, "#{name}:#{n + 1}: `%` separator has trailing whitespace"
        end
      end

      # -1 keeps empty entries, so a stray `%` at the end is caught too.
      entries = File.read(file).strip.split(/^%$/, -1).map(&:strip)
      if (i = entries.index(&:empty?))
        raise Errors::FatalException, "#{name}:#{separator_line(file, i)}: empty fortune"
      end
      entries
    end

    # The line of the `%` after the i-th entry, or before it for the last one.
    def separator_line(file, i)
      lines = File.readlines(file, chomp: true)
      separators = lines.each_index.select { |n| lines[n] == '%' }.map { |n| n + 1 }
      separators[i] || separators[i - 1]
    end
  end
end
