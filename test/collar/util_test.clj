(ns collar.util-test
  (:require [collar.util :as u]
            [clojure.test :refer :all]))

(deftest timestamp
  (testing "exact time"
    (is (= "1984-04-20" (u/timestamp (* 451270000 1000)))))
  (testing "zero"
    (is (= "1970-01-01" (u/timestamp 0))))
  (testing "negative"
    (is (= "1969-12-31" (u/timestamp -1))))
  (testing "nil"
    (is (= String (type (u/timestamp nil))))))
