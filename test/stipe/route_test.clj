(ns stipe.route-test
  (:require [clojure.test :refer :all]
            [ring.mock.request :as mock]
            [stipe.core :as stipe]))

#_(deftest test-app
  (testing "main route"
    (let [response (stipe/app (mock/request :get "/"))]
      (is (= (:status response) 200))
      (is (= (:body response) "foo"))))

  (testing "not-found route"
    (let [response (stipe/app (mock/request :get "/invalid"))]
      (is (= (:status response) 404)))))
