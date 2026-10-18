# Render TikZ figures to inline SVG at build time.
#
#     {% tikz A two state automaton %}
#     \begin{tikzpicture}
#       \node[state] (a) {$q_0$};
#       \node[state, right=of a] (b) {$q_1$};
#       \draw[->] (a) -- node[above] {$\sigma$} (b);
#     \end{tikzpicture}
#     {% endtikz %}
#
# The body is placed verbatim in a LaTeX document (so `tikzcd` works as well as
# `tikzpicture`), compiled with `latex`, and converted with `dvisvgm`. The
# optional text after `tikz` becomes the figure's accessible label. Extra
# packages and libraries go in `tikz.preamble` in `_config.yml`.
#
# Results are stored in Jekyll's cache (.jekyll-cache/), so a figure is only
# compiled again when its source or the preamble changes.

require 'cgi/escape'
require 'digest'
require 'open3'
require 'tmpdir'

module Jekyll
  class TikzBlock < Liquid::Raw
    # Bump to invalidate every cached figure after changing the output format.
    VERSION = 1

    PREAMBLE = <<~'TEX'
      \def\pgfsysdriver{pgfsys-dvisvgm.def}
      \documentclass{article}
      \usepackage{amsmath,amssymb}
      \usepackage{tikz}
      \pagestyle{empty}
      \hoffset=-1in \voffset=-1in \oddsidemargin=0pt \topmargin=0pt
      \headheight=0pt \headsep=0pt \topskip=0pt \parindent=0pt
      \newsavebox\tikzfigure
      \newdimen\tikzborder \tikzborder=1pt
    TEX

    # Like the standalone class (which TeX Live's base packages lack): box the
    # figure at the page's top left corner and size the page to it, so the SVG
    # is cropped to the picture. TikZ leaves half of each line's width out of
    # its bounding box, so a 1pt border keeps the outer edges of strokes from
    # being clipped when the SVG is scaled down.
    DOCUMENT = <<~'TEX'
      \begin{document}
      \begin{lrbox}{\tikzfigure}%%
      %s%%
      \end{lrbox}%%
      \sbox{\tikzfigure}{%%
        \kern\tikzborder
        \vbox{\kern\tikzborder\copy\tikzfigure\kern\tikzborder}%%
        \kern\tikzborder}%%
      \special{papersize=\the\wd\tikzfigure,\the\ht\tikzfigure}%%
      \noindent\usebox{\tikzfigure}
      \end{document}
    TEX

    def render(context)
      site = context.registers[:site]
      page = context.registers[:page]
      label = @markup.strip
      tex = document(site.config.dig('tikz', 'preamble'), @body)
      key = Digest::SHA256.hexdigest("#{VERSION}\0#{tex}")

      svg = TikzBlock.cache.getset(key) { compile(tex, key[0, 8], page && page['path']) }
      svg = svg.sub('<svg ', %(<svg role="img" aria-label="#{CGI.escapeHTML(label)}" )) unless label.empty?

      # Kept on one line so kramdown treats it as a single raw HTML block.
      %(\n<figure class="tikz">#{svg}</figure>\n)
    end

    def self.cache
      @cache ||= Jekyll::Cache.new('Jekyll::TikzBlock')
    end

    protected

    # Raw rejects any text after the tag name, but here it's the label.
    def ensure_valid_markup(*); end

    private

    def document(preamble, body)
      "#{PREAMBLE}#{preamble}\n#{format(DOCUMENT, body.strip)}"
    end

    def compile(tex, id, path)
      Jekyll.logger.info 'TikZ:', "compiling figure #{id} in #{path}"
      Dir.mktmpdir('tikz') do |dir|
        File.write(File.join(dir, 'fig.tex'), tex)
        run(dir, path, 'latex', '-interaction=nonstopmode', '-halt-on-error', 'fig.tex') do
          # The error, from its `!` line through the `l.<n>` line and source.
          log = File.read(File.join(dir, 'fig.log')).lines.drop_while { |l| !l.start_with?('!') }
          log.first((log.index { |l| l =~ /\Al\.\d+/ } || 6) + 2).join
        end
        run(dir, path, 'dvisvgm', '--no-fonts', '--bbox=papersize', '--optimize', 'fig.dvi', '-o', 'fig.svg')
        tidy(File.read(File.join(dir, 'fig.svg')), id)
      end
    end

    def run(dir, path, *cmd)
      begin
        out, status = Open3.capture2e(*cmd, chdir: dir)
      rescue Errno::ENOENT
        raise Jekyll::Errors::FatalException,
              "TikZ: `#{cmd.first}` not found. Install TeX Live and dvisvgm to build #{path}"
      end
      return if status.success?

      detail = block_given? ? yield : out
      raise Jekyll::Errors::FatalException, "TikZ: #{cmd.first} failed in #{path}\n#{detail}"
    end

    def tidy(svg, id)
      svg = svg.sub(/\A.*?(?=<svg)/m, '') # Drop the XML declaration and comment.

      # dvisvgm names glyphs g0-97 etc. in every file, so give each figure its
      # own ids to keep figures on the same page from sharing glyphs.
      svg = svg.gsub(/\bid='([^']+)'/) { "id='tikz-#{id}-#{$1}'" }
               .gsub(/(href='#|url\(#)([^')]+)/) { "#{$1}tikz-#{id}-#{$2}" }

      # Draw black in the page's text color.
      svg = svg.gsub(/='#000'/, "='currentColor'")

      # Size in em, taking TeX's 10pt as 1em, so labels match the surrounding
      # text (and MathJax) at any font size.
      svg = svg.sub(/width='([\d.]+)pt' height='([\d.]+)pt'/) do
        format("width='%.4fem' height='%.4fem'", $1.to_f / 10, $2.to_f / 10)
      end

      svg.gsub(/\s*\n\s*/, ' ').strip
    end
  end
end

Liquid::Template.register_tag('tikz', Jekyll::TikzBlock)
