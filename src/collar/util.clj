(ns collar.util
  (:require [config.core :refer [env]]))

(def is-dev? (= (:env env) "development"))
(def is-next? (= (:env env) "next"))
(def is-master? (= (:env env) "master"))
