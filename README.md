# collar

### about

- purpose:
    - an exercise after http://clojure-doc.org/articles/tutorials/basic_web_development.html
    - a space to play around, test ideas, compile, and exhibit
- distribution:
    - [alocy.be](https://alocy.be)
- license:
    - [mit](https://raw.githubusercontent.com/agarick/collar/master/LICENSE)
- naming
    - "where 'the other head' rests"

### operation

- Place all [markdown](https://github.com/yogthos/markdown-clj) files (ie. blog posts) into a folder situated as `/db/`. The filenames should end in `.md` and the file contents can include [metadata](https://github.com/fletcher/MultiMarkdown/wiki/MultiMarkdown-Syntax-Guide#metadata), though only `Title`, `Date`, and `Keywords` are used.
- All other aspects (eg. the navigation bar, or the "pages" page) are hardcoded, most of them appearing in `/src/collar/page.clj`.
