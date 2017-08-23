(ns collar.util
  (:require [config.core :as c]))

(def site-path "https://alocy.be")
(def db-path "db/")

(def env (:env c/env))
(def is-dev? (= env "development"))
(def is-next? (= env "next"))
(def is-master? (= env "master"))
