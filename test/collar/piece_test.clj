(ns collar.piece-test
  (:require [collar.piece :as p]
            [collar.util :as u]
            [clojure.test :refer :all]))

(deftest create-tag
  (testing "nil class"
    (is (some? (p/create-tag "foo" nil))))
  (testing "nil name"
    (is (thrown? IllegalArgumentException (p/create-tag nil "foo"))))
  (testing "nil"
    (is (thrown? IllegalArgumentException (p/create-tag nil nil)))))

(deftest create-page
  (is (.contains (p/create-page {} "") "DOCTYPE"))
  (let [page (p/create-page {:short "bar baz"} "foo")]
    (is (.contains page (str "<title>bar baz - " u/site-name)))
    (is (.contains page (str "<h1 class=\"title\">bar baz</")))
    (is (.contains page (str "<div class=\"page-body\">foo</")))))
