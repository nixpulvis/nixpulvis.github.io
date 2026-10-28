# Publish every Markdown page's source next to its HTML, after Liquid runs but
# before kramdown, so `/ramblings/foo` is also readable at `/ramblings/foo.md`.
# Also write `/site.json`, listing the nav and every collection document, for
# clients like the terminal view to browse the site without scraping HTML.

module Jekyll
  module MarkdownSource
    # Front matter worth keeping in the published source.
    KEYS = %w[title date description draft status references].freeze

    def self.sources
      @sources ||= {}
    end

    # The path a document's Markdown is published at, e.g. `/ramblings/foo.md`
    # or `/fortunes/index.md`.
    def self.url(url)
      url += 'index' if url.end_with?('/')
      url.sub(/\.html\z/, '') + '.md'
    end

    def self.front_matter(doc)
      data = KEYS.each_with_object({}) do |key, h|
        h[key] = doc.data[key] unless doc.data[key].nil?
      end
      # Jekyll dates undated documents with the build time, which means nothing.
      if data['date'] == doc.site.time
        data.delete('date')
      elsif data['date'].respond_to?(:strftime)
        data['date'] = data['date'].strftime('%Y-%m-%d')
      end
      data.delete('draft') unless data['draft']
      data['collection'] = doc.collection.label if doc.respond_to?(:collection)
      data
    end

    # Renderer#convert runs right after Liquid, on the document's content
    # alone, so this is the one place the Liquid output is in hand. Excerpts
    # share their document's data but only hold its first paragraph.
    module Capture
      def convert(content)
        if document.data['markdown'] && !document.is_a?(Excerpt)
          MarkdownSource.sources[document] = content
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
      doc.data['markdown'] = MarkdownSource.url(doc.url)
    end
  end

  Hooks.register :site, :after_reset do
    MarkdownSource.sources.clear
  end

  Hooks.register :site, :post_write do |site|
    MarkdownSource.sources.each do |doc, content|
      front = MarkdownSource.front_matter(doc)
      body = front.empty? ? content : "#{front.to_yaml}---\n\n#{content}"
      path = site.in_dest_dir(doc.data['markdown'])
      FileUtils.mkdir_p(File.dirname(path))
      File.write(path, body)
    end

    pages = site.collections.values.flat_map(&:docs).select do |doc|
      MarkdownSource.sources.key?(doc)
    end
    index = {
      'title' => 'nixpulvis',
      'url' => site.config['url'],
      'baseurl' => site.config['baseurl'].to_s,
      'nav' => site.data['nav'].map do |item|
        item.merge('source' => MarkdownSource.url(item['url']))
      end,
      'pages' => pages.map do |doc|
        MarkdownSource.front_matter(doc).slice('title', 'date', 'collection', 'draft', 'status').merge(
          'url' => doc.url,
          'source' => doc.data['markdown'],
        )
      end,
    }
    File.write(site.in_dest_dir('site.json'), JSON.pretty_generate(index))
  end
end
