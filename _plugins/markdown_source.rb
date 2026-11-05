# Publish every Markdown page's source next to its HTML, after Liquid runs but
# before kramdown, so `/ramblings/foo` is also readable at `/ramblings/foo.md`.
# The page's own layouts are applied too, all but the site-wide default, so
# extras like a project's references come along.
# Also write `/site.json`, listing the nav and every collection document, for
# clients like the terminal view to browse the site without scraping HTML.

module Jekyll
  module MarkdownSource
    # Front matter worth keeping in the published source.
    KEYS = %w[title date description draft status references].freeze

    # The front matter each page in `site.json` carries.
    INDEX_KEYS = %w[title date collection draft].freeze

    # The site-wide layout, with the header and nav, left off the source.
    SITE_LAYOUT = 'default'.freeze

    # Layouts that show a word count under the title.
    WORD_COUNT_LAYOUTS = %w[rambling article].freeze

    # The filters the layouts count words with.
    module Filters
      extend Liquid::StandardFilters
      extend Jekyll::Filters
    end

    # Every page's source this build, by document.
    def self.sources
      @sources ||= {}
    end

    # The path a page at `url` publishes its Markdown at, e.g.
    # `/ramblings/foo.md` or `/fortunes/index.md`.
    def self.path_for(url)
      url += 'index' if url.end_with?('/')
      url.sub(/\.html\z/, '') + '.md'
    end

    # A page's Markdown, as Liquid left it, and the front matter it's
    # published with.
    class Source
      attr_reader :doc

      def initialize(doc, content)
        @doc = doc
        @content = content
      end

      def path
        doc.data['markdown']
      end

      def front_matter
        @front_matter ||= begin
          data = doc.data.slice(*KEYS).compact
          # Jekyll dates undated documents with the build time, or with the
          # file's modified time for drafts, neither of which means anything.
          if [doc.site.time, File.mtime(doc.path)].include?(data['date'])
            data.delete('date')
          elsif data['date'].respond_to?(:strftime)
            data['date'] = data['date'].strftime('%Y-%m-%d')
          end
          data.delete('draft') unless data['draft']
          data['words'] = word_count if WORD_COUNT_LAYOUTS.include?(doc.data['layout'])
          data['collection'] = doc.collection.label if doc.respond_to?(:collection)
          data
        end
      end

      def write
        file = doc.site.in_dest_dir(path)
        FileUtils.mkdir_p(File.dirname(file))
        File.write(file, body)
      end

      # This page's entry in `site.json`.
      def index_entry
        front_matter.slice(*INDEX_KEYS).merge('url' => doc.url)
      end

      private

      def body
        content = in_layouts
        return content if front_matter.empty?

        "#{front_matter.to_yaml}---\n\n#{content}"
      end

      # The content wrapped in the page's layouts, up to the site-wide one.
      def in_layouts
        site = doc.site
        payload = site.site_payload
        payload['page'] = doc.to_liquid
        info = { registers: { site: site, page: payload['page'] } }
        content = @content
        name = doc.data['layout']
        while (layout = site.layouts[name]) && name != SITE_LAYOUT
          payload['content'] = content
          payload['layout'] = layout.data
          template = site.liquid_renderer.file(layout.path).parse(layout.content)
          content = template.render!(payload, info)
          name = layout.data['layout']
        end
        content
      end

      # Counted the way the layouts count, from the rendered content.
      def word_count
        Filters.number_of_words(Filters.strip_html(doc.content))
      end
    end

    # `site.json`, the nav and every collection document with a source.
    class Index
      def initialize(site)
        @site = site
      end

      def write
        File.write(@site.in_dest_dir('site.json'), JSON.pretty_generate(to_h))
      end

      private

      def to_h
        {
          'title' => @site.config['title'],
          'url' => @site.config['url'],
          'baseurl' => @site.config['baseurl'].to_s,
          'nav' => @site.data['nav'],
          'pages' => pages.map(&:index_entry),
        }
      end

      def pages
        @site.collections.values.flat_map(&:docs).filter_map do |doc|
          MarkdownSource.sources[doc]
        end
      end
    end

    # Renderer#convert runs right after Liquid, on the document's content
    # alone, so this is the one place the Liquid output is in hand. Excerpts
    # share their document's data but only hold its first paragraph.
    module Capture
      def convert(content)
        if document.data['markdown'] && !document.is_a?(Excerpt)
          MarkdownSource.sources[document] = Source.new(document, content)
        end
        super
      end
    end
  end

  Renderer.prepend(MarkdownSource::Capture)

  # Mark which pages get a published source, and where, so layouts can link
  # it as `page.markdown`.
  Hooks.register :site, :pre_render do |site|
    (site.pages + site.documents).each do |doc|
      next unless doc.extname == '.md' && doc.output_ext == '.html'
      doc.data['markdown'] = MarkdownSource.path_for(doc.url)
    end
  end

  Hooks.register :site, :after_reset do
    MarkdownSource.sources.clear
  end

  Hooks.register :site, :post_write do |site|
    MarkdownSource.sources.each_value(&:write)
    MarkdownSource::Index.new(site).write
  end
end
