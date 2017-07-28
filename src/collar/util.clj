(ns collar.util
  (:require [config.core :as c]))

(def site-path "https://alocy.be") ; base uri
(def db-path "db/") ; directory holding raw page files

; check current stage
(def is-dev? (= (:env c/env) "development"))
(def is-next? (= (:env c/env) "next"))
(def is-master? (= (:env c/env) "master"))
