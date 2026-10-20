# Read the fortune databases in fortunes/ into `site.data.fortunes`, newest
# year first, for fortunes.md to render.
#
#     [{ "year" => "2024", "path" => "/fortunes/2024",
#        "dat" => "/fortunes/2024.dat", "entries" => ["Loose cannons..."] }]
#
# Each file is in strfile's format: entries separated by a line holding only
# `%`. An empty entry fails the build, since `fortune` would print nothing for
# it. `dat` is nil when the file hasn't been compiled (see `make fortunes`).

module Jekyll
  class FortunesGenerator < Generator
    def generate(site)
      dir = File.join(site.source, 'fortunes')
      files = Dir.glob(File.join(dir, '[0-9][0-9][0-9][0-9]')).sort.reverse

      site.data['fortunes'] = files.map do |file|
        year = File.basename(file)
        {
          'year' => year,
          'path' => "/fortunes/#{year}",
          'dat' => (File.exist?("#{file}.dat") ? "/fortunes/#{year}.dat" : nil),
          'entries' => parse(file, "fortunes/#{year}"),
        }
      end
    end

    private

    def parse(file, name)
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
