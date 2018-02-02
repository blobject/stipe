# collar

![alocybe](https://alocy.be/img/alocybe-64.png "agaric")

### about

- purpose:
    - personal blogging platform
    - an exercise after http://clojure-doc.org/articles/tutorials/basic_web_development.html (@ july 2017)
- deployed at:
    - [alocy.be](https://alocy.be)
- license:
    - [mit](https://raw.githubusercontent.com/agarick/collar/master/LICENSE): source code
    - [cc by-nc-sa 4.0](https://alocy.be/about): superposed content
- naming
    - "where 'the other head' rests"

### operation

- Place all [markdown](https://github.com/yogthos/markdown-clj) files (ie. your original posts, entries, and pages) into a folder situated as `/db/`. The filenames should end in `.md` and the file contents can include [metadata](https://github.com/fletcher/MultiMarkdown/wiki/MultiMarkdown-Syntax-Guide#metadata), of which only `Title`, `Date`, and `Keywords` are used.
- Create `/config/master/config.edn` with `:env` and `:port` entries.
- Run: `lein with-profile master run`
