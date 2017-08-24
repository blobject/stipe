(ns collar.page-test
  (:require [collar.page :as p]
            [collar.util :as u]
            [clojure.test :refer :all]))

(deftest timestamp
  (testing "exact time"
    (is (= "1984-04-20" (p/timestamp (* 451270000 1000)))))
  (testing "zero"
    (is (= "1970-01-01" (p/timestamp 0))))
  (testing "negative"
    (is (= "1969-12-31" (p/timestamp -1))))
  (testing "nil"
    (is (= String (type (p/timestamp nil))))))

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
