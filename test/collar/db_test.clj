(ns collar.db-test
  (:require [collar.db :refer :all]
            [datomic.api :as d]
            [expectations :refer :all]))

(expect #{["foo"]}
        (with-redefs [conn (fresh-db)]
          (do
            (add-page "foo")
            (find-pages))))

(expect #{["footitle"]}
        (with-redefs [conn (fresh-db)]
          (do
            (add-page "foo")
            (add-title-to-page "foo" "footitle")
            (find-page-title "foo"))))

(expect #{["qux"] ["wharbly"]}
        (with-redefs [conn (fresh-db)]
          (do
            (add-tag "qux")
            (add-tag "wharbly")
            (find-tags))))

(expect #{["qux"]}
        (with-redefs [conn (fresh-db)]
          (do
            (add-page "foo")
            (add-tag "qux")
            (add-tag-to-page "foo" "qux")
            (find-page-tags "foo"))))
